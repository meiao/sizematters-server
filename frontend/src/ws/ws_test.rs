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

//! `process_message` routing. `RoomJoined` and `Randomized` are not covered: they call browser
//! APIs (localStorage, speech synthesis) that don't exist outside wasm.

use super::*;
use crate::rooms::jyanken::JyankenStore;
use crate::rooms::sizing::VoteStore;
use sizematters_shared::messages::sizing::{SizingResponseMessage, VotingSystem};
use sizematters_shared::UserData;

struct Fixture {
    owner: Owner,
    room_store: RoomStore,
    user_store: UserStore,
    rooms: RoomStores,
    last_error: RwSignal<Option<String>>,
}

impl Fixture {
    /// Already in sizing room "s" and jyanken room "j", each with "u1".
    fn new() -> Self {
        let owner = Owner::new();
        owner.with(|| {
            let rooms = RoomStores::new();
            rooms.provide();
            let room_store = RoomStore::new();
            for (name, room_type) in [("s", RoomType::Sizing), ("j", RoomType::Jyanken)] {
                room_store.room_joined(
                    name.to_string(),
                    String::new(),
                    room_type,
                    vec![user("u1", "Ann")],
                );
                rooms.room_joined(room_type, name, &["u1".to_string()]);
            }
            Fixture {
                owner: owner.clone(),
                room_store,
                user_store: UserStore::new(),
                rooms,
                last_error: RwSignal::new(None),
            }
        })
    }

    fn process(&self, msg: ClientResponseMessage) {
        process_message(
            msg,
            self.room_store,
            self.user_store,
            self.rooms,
            self.last_error,
        );
    }

    fn vote_store(&self) -> VoteStore {
        self.owner.with(expect_context::<VoteStore>)
    }

    fn jyanken_store(&self) -> JyankenStore {
        self.owner.with(expect_context::<JyankenStore>)
    }

    fn room_users(&self, room: &str) -> Vec<UserData> {
        self.room_store
            .rooms_signal()
            .get_untracked()
            .into_iter()
            .find(|r| r.room_name == room)
            .map(|r| r.users)
            .unwrap_or_default()
    }

    fn error(&self) -> Option<String> {
        self.last_error.get_untracked()
    }
}

fn user(user_id: &str, name: &str) -> UserData {
    let mut user = UserData::new(user_id.to_string(), String::new());
    user.name = name.to_string();
    user
}

#[test]
fn own_data_sets_our_identity() {
    let fixture = Fixture::new();

    fixture.process(ClientResponseMessage::OwnData {
        user: user("me", "Me"),
    });

    assert_eq!(fixture.user_store.own_user_id_untracked(), "me");
}

#[test]
fn user_joined_reaches_every_store() {
    let fixture = Fixture::new();

    fixture.process(ClientResponseMessage::UserJoined {
        room_name: "s".to_string(),
        user: user("u2", "Bob"),
    });

    assert_eq!(
        fixture.user_store.user_untracked("u2"),
        Some(user("u2", "Bob"))
    );
    assert!(fixture.room_users("s").contains(&user("u2", "Bob")));
    assert!(fixture.vote_store().is_spectator("s", "u2"));
    assert!(!fixture.room_users("j").contains(&user("u2", "Bob")));
}

#[test]
fn user_joined_a_jyanken_room_reaches_its_store() {
    let fixture = Fixture::new();

    fixture.process(ClientResponseMessage::UserJoined {
        room_name: "j".to_string(),
        user: user("u2", "Bob"),
    });

    assert!(fixture.jyanken_store().state("j").spectators.contains("u2"));
    assert!(!fixture.vote_store().is_spectator("j", "u2"));
}

#[test]
fn user_joined_an_unknown_room_only_updates_the_user() {
    let fixture = Fixture::new();

    fixture.process(ClientResponseMessage::UserJoined {
        room_name: "nope".to_string(),
        user: user("u2", "Bob"),
    });

    assert_eq!(
        fixture.user_store.user_untracked("u2"),
        Some(user("u2", "Bob"))
    );
    assert_eq!(fixture.room_store.rooms_signal().get_untracked().len(), 2);
}

#[test]
fn user_left_reaches_the_room_stores() {
    let fixture = Fixture::new();

    fixture.process(ClientResponseMessage::UserLeft {
        room_name: "s".to_string(),
        user_id: "u1".to_string(),
    });

    assert!(fixture.room_users("s").is_empty());
    assert!(!fixture.vote_store().is_spectator("s", "u1"));
    assert_eq!(fixture.room_users("j").len(), 1);
}

#[test]
fn user_updated_reaches_every_room() {
    let fixture = Fixture::new();

    fixture.process(ClientResponseMessage::UserUpdated {
        user: user("u1", "Annie"),
    });

    assert_eq!(
        fixture.user_store.user_untracked("u1"),
        Some(user("u1", "Annie"))
    );
    assert_eq!(fixture.room_users("s"), vec![user("u1", "Annie")]);
    assert_eq!(fixture.room_users("j"), vec![user("u1", "Annie")]);
}

#[test]
fn room_messages_go_to_the_rooms_handler() {
    let fixture = Fixture::new();
    let payload = serde_json::to_string(&SizingResponseMessage::VotingSystemChanged {
        new_system: VotingSystem::FistToFive,
    })
    .unwrap();

    fixture.process(ClientResponseMessage::RoomMessage {
        room_name: "s".to_string(),
        payload,
    });

    assert_eq!(
        fixture.vote_store().voting_system("s"),
        VotingSystem::FistToFive
    );
}

#[test]
fn room_message_errors_are_surfaced() {
    let fixture = Fixture::new();
    let payload = serde_json::to_string(&SizingResponseMessage::VotingOver).unwrap();

    fixture.process(ClientResponseMessage::RoomMessage {
        room_name: "s".to_string(),
        payload,
    });

    assert_eq!(fixture.error().as_deref(), Some("Voting is already over."));
}

#[test]
fn room_messages_for_unknown_rooms_are_ignored() {
    let fixture = Fixture::new();
    let payload = serde_json::to_string(&SizingResponseMessage::VotingOver).unwrap();

    fixture.process(ClientResponseMessage::RoomMessage {
        room_name: "nope".to_string(),
        payload,
    });

    assert_eq!(fixture.error(), None);
}

#[test]
fn server_errors_are_shown_to_the_user() {
    let cases = [
        (
            ClientResponseMessage::InvalidRoomName,
            "Invalid room name. Use 1-50 characters: letters, digits, hyphens, underscores.",
        ),
        (
            ClientResponseMessage::WrongPassword {
                room_name: "team".to_string(),
            },
            "Wrong password for room 'team'.",
        ),
        (
            ClientResponseMessage::AlreadyInRoom {
                room_name: "team".to_string(),
            },
            "You are already in room 'team'.",
        ),
        (
            ClientResponseMessage::Error {
                msg: "Room not found.".to_string(),
            },
            "Room not found.",
        ),
    ];
    for (msg, expected) in cases {
        let fixture = Fixture::new();
        fixture.process(msg);
        assert_eq!(fixture.error().as_deref(), Some(expected));
    }
}
