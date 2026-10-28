/*
 * SizeMatters - a ticket sizing util
 * Copyright (C) 2026 Andre Onuki
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use super::*;
use crate::actors::rooms::test_support::*;
use actix::Addr;
use serde::Deserialize;
use std::sync::Mutex;

const SECRET_MD5: &str = "5ebe2294ecd0e0f08eab7690d2a6ee69";

/// What the room's state looked like when a hook was called.
#[derive(Debug, Clone, PartialEq)]
struct Call {
    hook: &'static str,
    user_id: String,
    user_count: usize,
    spectator_count: usize,
    is_spectator: bool,
}

#[derive(Deserialize)]
enum TestMsg {
    Spectate,
    Play,
}

/// Records every hook call, and lets tests change the spectators through messages.
struct RecordingBehavior {
    calls: Arc<Mutex<Vec<Call>>>,
}

impl RecordingBehavior {
    fn record(&self, hook: &'static str, user_id: &Arc<String>, room: &RoomState) {
        self.calls.lock().unwrap().push(Call {
            hook,
            user_id: (**user_id).clone(),
            user_count: room.user_count(),
            spectator_count: room.spectator_count(),
            is_spectator: room.is_spectator(user_id),
        });
    }
}

impl RoomBehavior<TestMsg> for RecordingBehavior {
    fn on_join_room(&mut self, user_id: &Arc<String>, room: &mut RoomState) {
        self.record("join", user_id, room);
    }

    fn on_leave_room(&mut self, user_id: &Arc<String>, room: &mut RoomState) {
        self.record("leave", user_id, room);
    }

    fn on_room_closing(&mut self, room: &mut RoomState) {
        self.record("closing", &Arc::new(String::new()), room);
    }

    fn on_specific_message(&mut self, user_id: Arc<String>, msg: TestMsg, room: &mut RoomState) {
        match msg {
            TestMsg::Spectate => room.add_spectator(user_id.clone()),
            TestMsg::Play => room.remove_spectator(user_id.clone()),
        }
        self.record("message", &user_id, room);
    }
}

struct Fixture {
    manager: ManagerProbe,
    room: Addr<Room<TestMsg>>,
    calls: Arc<Mutex<Vec<Call>>>,
}

impl Fixture {
    fn new() -> Self {
        let manager = ManagerProbe::start();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let behavior = RecordingBehavior {
            calls: calls.clone(),
        };
        let room = start_room(&manager, RoomType::Sizing, Box::new(behavior));
        Fixture {
            manager,
            room,
            calls,
        }
    }

    /// Joins `user_id` with the right password and returns its probe, with nothing received yet.
    async fn join(&self, user_id: &str) -> ClientProbe {
        let client = ClientProbe::start();
        self.send(join_msg(user_id, PASSWORD, &client)).await;
        client.take().await;
        client
    }

    async fn send(&self, msg: RoomMessages) {
        self.room.send(msg).await.unwrap();
    }

    fn take_calls(&self) -> Vec<Call> {
        std::mem::take(&mut self.calls.lock().unwrap())
    }
}

fn user_ids(users: &[UserData]) -> Vec<String> {
    let mut ids: Vec<String> = users.iter().map(|u| u.user_id.clone()).collect();
    ids.sort();
    ids
}

#[test]
fn compute_password_hashes_plaintext_with_md5() {
    assert_eq!(*compute_password(arc("secret"), false), SECRET_MD5);
    assert_eq!(
        *compute_password(arc(""), false),
        "d41d8cd98f00b204e9800998ecf8427e"
    );
}

#[test]
fn compute_password_keeps_hashes_as_they_are() {
    assert_eq!(*compute_password(arc(SECRET_MD5), true), SECRET_MD5);
    assert_eq!(*compute_password(arc("anything"), true), "anything");
}

#[test]
fn user_info_round_trips_user_data() {
    let mut data = user("u1");
    data.avatar_style = AvatarStyle::Squircles;
    let info = UserInfo::from(data.clone());
    assert_eq!(UserData::from(&info), data);
}

#[actix_rt::test]
async fn first_user_joins_as_spectator() {
    let fixture = Fixture::new();
    let client = ClientProbe::start();

    fixture.send(join_msg("u1", PASSWORD, &client)).await;

    let msgs = client.take().await;
    assert_eq!(msgs.len(), 2, "{msgs:?}");
    assert!(matches!(
        &msgs[0],
        ClientResponseMessage::UserJoined { room_name, user: joined }
            if room_name == ROOM && *joined == user("u1")
    ));
    match &msgs[1] {
        ClientResponseMessage::RoomJoined {
            room_name,
            hashed_password,
            room_type,
            users,
        } => {
            assert_eq!(room_name, ROOM);
            assert_eq!(hashed_password, SECRET_MD5);
            assert_eq!(*room_type, RoomType::Sizing);
            assert_eq!(users, &vec![user("u1")]);
        }
        other => panic!("expected RoomJoined, got {other:?}"),
    }

    assert_eq!(
        fixture.take_calls(),
        vec![Call {
            hook: "join",
            user_id: "u1".to_string(),
            user_count: 1,
            spectator_count: 1,
            is_spectator: true,
        }]
    );

    let manager_msgs = fixture.manager.take().await;
    assert_eq!(manager_msgs.len(), 1);
    assert!(matches!(
        &manager_msgs[0],
        RoomMessages::UserJoined { user_name, room_name }
            if **user_name == "u1" && **room_name == ROOM
    ));
}

#[actix_rt::test]
async fn joining_user_is_announced_to_everyone_and_sees_everyone() {
    let fixture = Fixture::new();
    let first = fixture.join("u1").await;
    let second = ClientProbe::start();

    fixture.send(join_msg("u2", PASSWORD, &second)).await;

    let first_msgs = first.take().await;
    assert_eq!(first_msgs.len(), 1);
    assert!(matches!(
        &first_msgs[0],
        ClientResponseMessage::UserJoined { user, .. } if user.user_id == "u2"
    ));

    let second_msgs = second.take().await;
    let users = second_msgs
        .iter()
        .find_map(|m| match m {
            ClientResponseMessage::RoomJoined { users, .. } => Some(users.clone()),
            _ => None,
        })
        .expect("RoomJoined");
    assert_eq!(user_ids(&users), vec!["u1", "u2"]);
}

#[actix_rt::test]
async fn hashed_password_from_a_room_link_is_accepted() {
    let fixture = Fixture::new();
    let client = ClientProbe::start();

    fixture
        .send(RoomMessages::JoinRoom {
            room_name: arc(ROOM),
            password: arc(SECRET_MD5),
            password_is_hash: true,
            user: user("u1"),
            recipient: client.recipient(),
        })
        .await;

    let msgs = client.take().await;
    assert!(msgs
        .iter()
        .any(|m| matches!(m, ClientResponseMessage::RoomJoined { .. })));
}

#[actix_rt::test]
async fn wrong_password_is_rejected() {
    let fixture = Fixture::new();
    let member = fixture.join("u1").await;
    fixture.take_calls();
    let intruder = ClientProbe::start();

    fixture.send(join_msg("u2", "guess", &intruder)).await;

    let msgs = intruder.take().await;
    assert_eq!(msgs.len(), 1);
    assert!(matches!(
        &msgs[0],
        ClientResponseMessage::WrongPassword { room_name } if room_name == ROOM
    ));
    assert!(member.take().await.is_empty());
    assert!(fixture.take_calls().is_empty());
}

#[actix_rt::test]
async fn hash_of_the_wrong_password_is_rejected() {
    let fixture = Fixture::new();
    let client = ClientProbe::start();

    fixture
        .send(RoomMessages::JoinRoom {
            room_name: arc(ROOM),
            password: arc("secret"),
            // "secret" is not the hash of "secret"
            password_is_hash: true,
            user: user("u1"),
            recipient: client.recipient(),
        })
        .await;

    let msgs = client.take().await;
    assert!(matches!(
        &msgs[..],
        [ClientResponseMessage::WrongPassword { .. }]
    ));
}

#[actix_rt::test]
async fn joining_twice_is_rejected() {
    let fixture = Fixture::new();
    let client = fixture.join("u1").await;
    fixture.take_calls();

    fixture.send(join_msg("u1", PASSWORD, &client)).await;

    let msgs = client.take().await;
    assert_eq!(msgs.len(), 1);
    assert!(matches!(
        &msgs[0],
        ClientResponseMessage::AlreadyInRoom { room_name } if room_name == ROOM
    ));
    assert!(fixture.take_calls().is_empty());
}

#[actix_rt::test]
async fn leaving_user_is_announced_to_everyone_including_them() {
    let fixture = Fixture::new();
    let staying = fixture.join("u1").await;
    let leaving = fixture.join("u2").await;
    staying.take().await;
    fixture.take_calls();

    fixture
        .send(RoomMessages::UserLeft { user_id: arc("u2") })
        .await;

    for client in [&staying, &leaving] {
        let msgs = client.take().await;
        assert_eq!(msgs.len(), 1, "{msgs:?}");
        assert!(matches!(
            &msgs[0],
            ClientResponseMessage::UserLeft { room_name, user_id }
                if room_name == ROOM && user_id == "u2"
        ));
    }
    assert_eq!(
        fixture.take_calls(),
        vec![Call {
            hook: "leave",
            user_id: "u2".to_string(),
            user_count: 1,
            spectator_count: 1,
            is_spectator: false,
        }]
    );
    assert!(fixture.room.connected());
}

#[actix_rt::test]
async fn room_closes_when_the_last_user_leaves() {
    let fixture = Fixture::new();
    fixture.join("u1").await;
    fixture.manager.take().await;
    fixture.take_calls();

    fixture
        .send(RoomMessages::UserLeft { user_id: arc("u1") })
        .await;

    let hooks: Vec<&str> = fixture.take_calls().iter().map(|c| c.hook).collect();
    assert_eq!(hooks, vec!["leave", "closing"]);

    let manager_msgs = fixture.manager.take().await;
    assert_eq!(manager_msgs.len(), 1);
    assert!(matches!(
        &manager_msgs[0],
        RoomMessages::RoomClosing { room_name } if **room_name == ROOM
    ));

    for _ in 0..100 {
        if !fixture.room.connected() {
            return;
        }
        actix_rt::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("room actor should have stopped");
}

#[actix_rt::test]
async fn unknown_user_leaving_changes_nothing() {
    let fixture = Fixture::new();
    let member = fixture.join("u1").await;
    fixture.take_calls();

    fixture
        .send(RoomMessages::UserLeft {
            user_id: arc("ghost"),
        })
        .await;

    assert!(member.take().await.is_empty());
    assert!(fixture.take_calls().is_empty());
    assert!(fixture.room.connected());
}

#[actix_rt::test]
async fn user_update_is_broadcast_and_kept() {
    let fixture = Fixture::new();
    let first = fixture.join("u1").await;
    let second = fixture.join("u2").await;
    first.take().await;

    let mut renamed = user("u1");
    renamed.name = "Renamed".to_string();
    fixture
        .send(RoomMessages::UserUpdated {
            user: renamed.clone(),
        })
        .await;

    for client in [&first, &second] {
        let msgs = client.take().await;
        assert_eq!(msgs.len(), 1);
        assert!(matches!(
            &msgs[0],
            ClientResponseMessage::UserUpdated { user } if *user == renamed
        ));
    }

    // Newcomers get the updated data.
    let third = ClientProbe::start();
    fixture.send(join_msg("u3", PASSWORD, &third)).await;
    let users = third
        .take()
        .await
        .into_iter()
        .find_map(|m| match m {
            ClientResponseMessage::RoomJoined { users, .. } => Some(users),
            _ => None,
        })
        .unwrap();
    assert!(users.contains(&renamed));
}

#[actix_rt::test]
async fn update_for_a_user_not_in_the_room_asks_the_manager_to_remove_them() {
    let fixture = Fixture::new();
    let member = fixture.join("u1").await;
    fixture.manager.take().await;

    fixture
        .send(RoomMessages::UserUpdated {
            user: user("ghost"),
        })
        .await;

    assert!(member.take().await.is_empty());
    let manager_msgs = fixture.manager.take().await;
    assert_eq!(manager_msgs.len(), 1);
    assert!(matches!(
        &manager_msgs[0],
        RoomMessages::LeaveRoom { room_name, user_id }
            if **room_name == ROOM && **user_id == "ghost"
    ));
}

#[actix_rt::test]
async fn specific_messages_are_parsed_and_given_to_the_behavior() {
    let fixture = Fixture::new();
    fixture.join("u1").await;
    fixture.take_calls();

    fixture.send(specific_msg("u1", r#""Play""#)).await;
    fixture.send(specific_msg("u1", r#""Spectate""#)).await;

    let calls = fixture.take_calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].hook, "message");
    assert!(!calls[0].is_spectator);
    assert_eq!(calls[0].spectator_count, 0);
    assert!(calls[1].is_spectator);
    assert_eq!(calls[1].spectator_count, 1);
}

#[actix_rt::test]
async fn unreadable_specific_message_is_reported_to_the_sender() {
    let fixture = Fixture::new();
    let sender = fixture.join("u1").await;
    let other = fixture.join("u2").await;
    sender.take().await;
    fixture.take_calls();

    fixture.send(specific_msg("u1", r#""Dance""#)).await;

    assert_eq!(
        errors(&sender.take().await),
        vec!["Unable to read message."]
    );
    assert!(other.take().await.is_empty());
    assert!(fixture.take_calls().is_empty());
}

#[actix_rt::test]
async fn unreachable_client_is_reported_to_the_manager() {
    let fixture = Fixture::new();
    let gone = fixture.join("u1").await;
    fixture.manager.take().await;
    gone.stop().await;

    // Joining notifies everyone, including the unreachable client.
    fixture.join("u2").await;

    let manager_msgs = fixture.manager.take().await;
    assert!(
        manager_msgs
            .iter()
            .any(|m| matches!(m, RoomMessages::UserLeft { user_id } if **user_id == "u1")),
        "manager should be told u1 is gone"
    );
}
