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

//! These tests run the real rooms behind the manager, so responses arrive asynchronously
//! (manager -> room -> client) and are awaited with `take_until`.

use super::*;
use crate::actors::rooms::test_support::*;
use sizematters_shared::messages::sizing::SizingResponseMessage;
use sizematters_shared::RoomType;

struct Fixture {
    manager: Addr<RoomManagerActor>,
}

impl Fixture {
    fn new() -> Self {
        Fixture {
            manager: RoomManagerActor::new().start(),
        }
    }

    async fn send(&self, msg: RoomMessages) {
        self.manager.send(msg).await.unwrap();
    }

    async fn create(
        &self,
        room: &str,
        password: &str,
        room_type: RoomType,
        user_id: &str,
        client: &ClientProbe,
    ) {
        self.send(RoomMessages::CreateRoom {
            room_name: arc(room),
            password: arc(password),
            password_is_hash: false,
            room_type,
            user: user(user_id),
            recipient: client.recipient(),
        })
        .await;
    }

    async fn join(&self, room: &str, password: &str, user_id: &str, client: &ClientProbe) {
        self.send(RoomMessages::JoinRoom {
            room_name: arc(room),
            password: arc(password),
            password_is_hash: false,
            user: user(user_id),
            recipient: client.recipient(),
        })
        .await;
    }

    /// Creates (or joins) `room` as `user_id` and waits until the room confirmed it.
    async fn enter(&self, room: &str, user_id: &str, client: &ClientProbe) -> RoomType {
        self.create(room, PASSWORD, RoomType::Sizing, user_id, client)
            .await;
        let msgs = client.take_until(|msgs| joined(msgs).is_some()).await;
        joined(&msgs).unwrap()
    }
}

/// The room type of the first `RoomJoined` among `msgs`.
fn joined(msgs: &[ClientResponseMessage]) -> Option<RoomType> {
    msgs.iter().find_map(|m| match m {
        ClientResponseMessage::RoomJoined { room_type, .. } => Some(*room_type),
        _ => None,
    })
}

fn has_error(msgs: &[ClientResponseMessage], text: &str) -> bool {
    errors(msgs).iter().any(|e| e == text)
}

fn user_left(msgs: &[ClientResponseMessage], room: &str, user: &str) -> bool {
    msgs.iter().any(|m| {
        matches!(m, ClientResponseMessage::UserLeft { room_name, user_id }
            if room_name == room && user_id == user)
    })
}

fn invalid_names() -> Vec<String> {
    vec![
        String::new(),
        "with space".to_string(),
        "émile".to_string(),
        "../etc".to_string(),
        "a".repeat(51),
    ]
}

#[actix_rt::test]
async fn creating_a_room_with_an_invalid_name_is_refused() {
    let fixture = Fixture::new();
    for name in invalid_names() {
        let client = ClientProbe::start();
        fixture
            .create(&name, PASSWORD, RoomType::Sizing, "u1", &client)
            .await;
        let msgs = client.take().await;
        assert!(
            matches!(msgs[..], [ClientResponseMessage::InvalidRoomName]),
            "{name:?}: {msgs:?}"
        );
    }
}

#[actix_rt::test]
async fn joining_a_room_with_an_invalid_name_is_refused() {
    let fixture = Fixture::new();
    for name in invalid_names() {
        let client = ClientProbe::start();
        fixture.join(&name, PASSWORD, "u1", &client).await;
        let msgs = client.take().await;
        assert!(
            matches!(msgs[..], [ClientResponseMessage::InvalidRoomName]),
            "{name:?}: {msgs:?}"
        );
    }
}

#[actix_rt::test]
async fn room_names_use_letters_digits_hyphens_and_underscores() {
    let fixture = Fixture::new();
    for (i, name) in ["a", "Team_A-9", &"z".repeat(50)].iter().enumerate() {
        let client = ClientProbe::start();
        fixture.enter(name, &format!("u{i}"), &client).await;
    }
}

#[actix_rt::test]
async fn joining_an_unknown_room_is_an_error() {
    let fixture = Fixture::new();
    let client = ClientProbe::start();

    fixture.join("nowhere", PASSWORD, "u1", &client).await;

    assert!(has_error(&client.take().await, "Room not found."));
}

#[actix_rt::test]
async fn created_room_has_the_requested_type() {
    let fixture = Fixture::new();
    let client = ClientProbe::start();

    fixture
        .create("games", PASSWORD, RoomType::Jyanken, "u1", &client)
        .await;

    let msgs = client.take_until(|msgs| joined(msgs).is_some()).await;
    assert_eq!(joined(&msgs), Some(RoomType::Jyanken));
}

#[actix_rt::test]
async fn others_can_join_a_created_room() {
    let fixture = Fixture::new();
    let owner = ClientProbe::start();
    let guest = ClientProbe::start();
    fixture.enter("team", "u1", &owner).await;

    fixture.join("team", PASSWORD, "u2", &guest).await;

    guest.take_until(|msgs| joined(msgs).is_some()).await;
    owner
        .take_until(|msgs| {
            msgs.iter().any(|m| {
                matches!(m, ClientResponseMessage::UserJoined { user, .. } if user.user_id == "u2")
            })
        })
        .await;
}

#[actix_rt::test]
async fn creating_an_existing_room_joins_it_and_keeps_its_type() {
    let fixture = Fixture::new();
    let owner = ClientProbe::start();
    let guest = ClientProbe::start();
    fixture.enter("team", "u1", &owner).await;

    // e.g. a client restoring its saved rooms
    fixture
        .create("team", PASSWORD, RoomType::Jyanken, "u2", &guest)
        .await;

    let msgs = guest.take_until(|msgs| joined(msgs).is_some()).await;
    assert_eq!(joined(&msgs), Some(RoomType::Sizing));
}

#[actix_rt::test]
async fn creating_an_existing_room_still_checks_the_password() {
    let fixture = Fixture::new();
    let owner = ClientProbe::start();
    let guest = ClientProbe::start();
    fixture.enter("team", "u1", &owner).await;

    fixture
        .create("team", "other", RoomType::Sizing, "u2", &guest)
        .await;

    let msgs = guest
        .take_until(|msgs| {
            msgs.iter()
                .any(|m| matches!(m, ClientResponseMessage::WrongPassword { .. }))
        })
        .await;
    assert!(joined(&msgs).is_none());
}

#[actix_rt::test]
async fn joining_a_room_twice_is_ignored() {
    let fixture = Fixture::new();
    let client = ClientProbe::start();
    fixture.enter("team", "u1", &client).await;

    fixture.join("team", PASSWORD, "u1", &client).await;

    assert!(client.take_after_settling().await.is_empty());
}

#[actix_rt::test]
async fn leaving_a_room_notifies_its_members() {
    let fixture = Fixture::new();
    let staying = ClientProbe::start();
    let leaving = ClientProbe::start();
    fixture.enter("team", "u1", &staying).await;
    fixture.enter("team", "u2", &leaving).await;

    fixture
        .send(RoomMessages::LeaveRoom {
            room_name: arc("team"),
            user_id: arc("u2"),
        })
        .await;

    staying
        .take_until(|msgs| user_left(msgs, "team", "u2"))
        .await;
    leaving
        .take_until(|msgs| user_left(msgs, "team", "u2"))
        .await;
}

#[actix_rt::test]
async fn user_can_rejoin_a_room_they_left() {
    let fixture = Fixture::new();
    let staying = ClientProbe::start();
    let leaving = ClientProbe::start();
    fixture.enter("team", "u1", &staying).await;
    fixture.enter("team", "u2", &leaving).await;
    fixture
        .send(RoomMessages::LeaveRoom {
            room_name: arc("team"),
            user_id: arc("u2"),
        })
        .await;
    leaving
        .take_until(|msgs| user_left(msgs, "team", "u2"))
        .await;

    fixture.enter("team", "u2", &leaving).await;
}

#[actix_rt::test]
async fn leaving_a_room_one_is_not_in_does_nothing() {
    let fixture = Fixture::new();
    let member = ClientProbe::start();
    fixture.enter("team", "u1", &member).await;

    fixture
        .send(RoomMessages::LeaveRoom {
            room_name: arc("team"),
            user_id: arc("u2"),
        })
        .await;
    fixture
        .send(RoomMessages::LeaveRoom {
            room_name: arc("elsewhere"),
            user_id: arc("u1"),
        })
        .await;

    assert!(member.take_after_settling().await.is_empty());
}

#[actix_rt::test]
async fn disconnected_user_leaves_every_room() {
    let fixture = Fixture::new();
    let gone = ClientProbe::start();
    let in_a = ClientProbe::start();
    let in_b = ClientProbe::start();
    fixture.enter("room-a", "u1", &gone).await;
    fixture.enter("room-b", "u1", &gone).await;
    fixture.enter("room-a", "u2", &in_a).await;
    fixture.enter("room-b", "u3", &in_b).await;

    fixture
        .send(RoomMessages::UserLeft { user_id: arc("u1") })
        .await;

    in_a.take_until(|msgs| user_left(msgs, "room-a", "u1"))
        .await;
    in_b.take_until(|msgs| user_left(msgs, "room-b", "u1"))
        .await;
}

#[actix_rt::test]
async fn user_updates_reach_every_room_of_the_user() {
    let fixture = Fixture::new();
    let client = ClientProbe::start();
    let in_a = ClientProbe::start();
    let in_b = ClientProbe::start();
    let elsewhere = ClientProbe::start();
    fixture.enter("room-a", "u1", &client).await;
    fixture.enter("room-b", "u1", &client).await;
    fixture.enter("room-a", "u2", &in_a).await;
    fixture.enter("room-b", "u3", &in_b).await;
    fixture.enter("room-c", "u4", &elsewhere).await;

    let mut renamed = user("u1");
    renamed.name = "Renamed".to_string();
    fixture
        .send(RoomMessages::UserUpdated {
            user: renamed.clone(),
        })
        .await;

    for member in [&in_a, &in_b] {
        member
            .take_until(|msgs| {
                msgs.iter().any(
                    |m| matches!(m, ClientResponseMessage::UserUpdated { user } if *user == renamed),
                )
            })
            .await;
    }
    assert!(elsewhere.take_after_settling().await.is_empty());
}

#[actix_rt::test]
async fn room_messages_are_forwarded_to_the_room() {
    let fixture = Fixture::new();
    let client = ClientProbe::start();
    fixture.enter("team", "u1", &client).await;

    fixture
        .send(RoomMessages::SpecificMessage {
            room_name: arc("team"),
            user_id: arc("u1"),
            payload: r#"{"type":"RegisterToVote"}"#.to_string(),
        })
        .await;

    client
        .take_until(|msgs| {
            msgs.iter().any(|m| match m {
                ClientResponseMessage::RoomMessage { payload, .. } => matches!(
                    serde_json::from_str(payload),
                    Ok(SizingResponseMessage::VoteStatus { spectators, .. }) if spectators.is_empty()
                ),
                _ => false,
            })
        })
        .await;
}

#[actix_rt::test]
async fn messages_to_unknown_rooms_are_dropped() {
    let fixture = Fixture::new();
    let client = ClientProbe::start();
    fixture.enter("team", "u1", &client).await;

    fixture
        .send(RoomMessages::SpecificMessage {
            room_name: arc("nowhere"),
            user_id: arc("u1"),
            payload: r#"{"type":"NewVote"}"#.to_string(),
        })
        .await;

    assert!(client.take_after_settling().await.is_empty());
}

#[actix_rt::test]
async fn empty_room_is_forgotten() {
    let fixture = Fixture::new();
    let client = ClientProbe::start();
    fixture.enter("team", "u1", &client).await;
    fixture
        .send(RoomMessages::LeaveRoom {
            room_name: arc("team"),
            user_id: arc("u1"),
        })
        .await;
    client
        .take_until(|msgs| user_left(msgs, "team", "u1"))
        .await;

    fixture.join("team", PASSWORD, "u2", &client).await;
    assert!(has_error(&client.take().await, "Room not found."));

    // The name is free again, for any type and password.
    fixture
        .create("team", "new password", RoomType::Jyanken, "u2", &client)
        .await;
    let msgs = client.take_until(|msgs| joined(msgs).is_some()).await;
    assert_eq!(joined(&msgs), Some(RoomType::Jyanken));
}

#[actix_rt::test]
async fn unreachable_client_is_removed_from_its_rooms() {
    let fixture = Fixture::new();
    let client = ClientProbe::start();
    let other = ClientProbe::start();
    fixture.enter("team", "u1", &client).await;
    fixture.enter("team", "u2", &other).await;
    let dead = ClientProbe::start();
    dead.stop().await;

    // Answering this request fails, which the manager takes as u1 being gone.
    fixture.join("bad name", PASSWORD, "u1", &dead).await;

    other.take_until(|msgs| user_left(msgs, "team", "u1")).await;
}
