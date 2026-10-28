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
use sizematters_shared::messages::jyanken::JyankenResponseMessage;
use sizematters_shared::messages::sizing::{SizingResponseMessage, VotingSystem};
use std::collections::HashMap;

fn ids(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

fn ctx() -> HandlerCtx {
    HandlerCtx {
        room_store: RoomStore::new(),
        user_store: UserStore::new(),
        last_error: RwSignal::new(None),
    }
}

fn has_sizing_room(stores: &RoomStores, room: &str) -> bool {
    stores
        .sizing
        .votes_signal()
        .get_untracked()
        .contains_key(room)
}

fn has_jyanken_room(stores: &RoomStores, room: &str) -> bool {
    stores.jyanken.state(room) != Default::default()
}

#[test]
fn room_joined_goes_to_the_store_of_the_room_type() {
    let stores = RoomStores::new();

    stores.room_joined(RoomType::Sizing, "s", &ids(&["u1"]));
    stores.room_joined(RoomType::Jyanken, "j", &ids(&["u1"]));

    assert!(has_sizing_room(&stores, "s"));
    assert!(!has_sizing_room(&stores, "j"));
    assert!(has_jyanken_room(&stores, "j"));
    assert!(!has_jyanken_room(&stores, "s"));
}

#[test]
fn user_joined_and_left_go_to_the_store_of_the_room_type() {
    let stores = RoomStores::new();
    stores.room_joined(RoomType::Sizing, "s", &[]);
    stores.room_joined(RoomType::Jyanken, "j", &[]);

    stores.user_joined(RoomType::Sizing, "s", "u2");
    stores.user_joined(RoomType::Jyanken, "j", "u3");
    assert!(stores.sizing.is_spectator("s", "u2"));
    assert!(stores.jyanken.state("j").spectators.contains("u3"));

    stores.user_left(RoomType::Sizing, "s", "u2");
    stores.user_left(RoomType::Jyanken, "j", "u3");
    assert!(!stores.sizing.is_spectator("s", "u2"));
    assert!(!stores.jyanken.state("j").spectators.contains("u3"));
}

#[test]
fn messages_go_to_the_handler_of_the_room_type() {
    let stores = RoomStores::new();
    stores.room_joined(RoomType::Sizing, "s", &[]);
    stores.room_joined(RoomType::Jyanken, "j", &[]);

    let sizing = serde_json::to_string(&SizingResponseMessage::VotingSystemChanged {
        new_system: VotingSystem::TShirt,
    })
    .unwrap();
    stores.handle_message(RoomType::Sizing, "s", &sizing, ctx());
    assert_eq!(stores.sizing.voting_system("s"), VotingSystem::TShirt);

    let jyanken = serde_json::to_string(&JyankenResponseMessage::GameResults {
        weapons: HashMap::new(),
        scoreboard: HashMap::new(),
    })
    .unwrap();
    stores.handle_message(RoomType::Jyanken, "j", &jyanken, ctx());
    assert_eq!(stores.jyanken.state("j").results, Some(HashMap::new()));
}

#[test]
fn payload_sent_to_the_wrong_room_type_is_ignored() {
    let stores = RoomStores::new();
    stores.room_joined(RoomType::Jyanken, "j", &[]);
    let ctx = ctx();

    let sizing = serde_json::to_string(&SizingResponseMessage::VotingOver).unwrap();
    stores.handle_message(RoomType::Jyanken, "j", &sizing, ctx);

    assert_eq!(ctx.last_error.get_untracked(), None);
}

#[test]
fn leave_forgets_the_room_whatever_its_type() {
    let stores = RoomStores::new();
    stores.room_joined(RoomType::Sizing, "s", &ids(&["u1"]));
    stores.room_joined(RoomType::Jyanken, "j", &ids(&["u1"]));

    stores.leave("s");
    stores.leave("j");

    assert!(!has_sizing_room(&stores, "s"));
    assert!(!has_jyanken_room(&stores, "j"));
}

#[test]
fn clear_forgets_every_room() {
    let stores = RoomStores::new();
    stores.room_joined(RoomType::Sizing, "s", &ids(&["u1"]));
    stores.room_joined(RoomType::Jyanken, "j", &ids(&["u1"]));

    stores.clear();

    assert!(!has_sizing_room(&stores, "s"));
    assert!(!has_jyanken_room(&stores, "j"));
}

#[test]
fn provide_makes_every_store_available_as_context() {
    let owner = Owner::new();
    owner.with(|| {
        let stores = RoomStores::new();
        stores.provide();
        stores.room_joined(RoomType::Sizing, "s", &ids(&["u1"]));
        stores.room_joined(RoomType::Jyanken, "j", &ids(&["u1"]));

        assert!(use_context::<RoomStores>().is_some());
        assert!(expect_context::<VoteStore>().is_spectator("s", "u1"));
        assert!(expect_context::<JyankenStore>()
            .state("j")
            .spectators
            .contains("u1"));
    });
}
