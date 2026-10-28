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
use crate::stores::{RoomStore, UserStore};
use sizematters_shared::messages::jyanken::JyankenWeapon;
use sizematters_shared::RoomType;
use std::collections::HashMap;

const ROOM: &str = "games";

struct Fixture {
    store: JyankenStore,
    ctx: HandlerCtx,
}

impl Fixture {
    /// We are in ROOM with "u2", and the randomizer picked u2.
    fn new() -> Self {
        let room_store = RoomStore::new();
        room_store.room_joined(ROOM.to_string(), String::new(), RoomType::Jyanken, vec![]);
        room_store.randomized(ROOM, "u2");
        let store = JyankenStore::new();
        store.room_joined(ROOM, &["me".to_string(), "u2".to_string()]);
        Fixture {
            store,
            ctx: HandlerCtx {
                room_store,
                user_store: UserStore::new(),
                last_error: RwSignal::new(None),
            },
        }
    }

    fn handle(&self, msg: JyankenResponseMessage) {
        let payload = serde_json::to_string(&msg).unwrap();
        handle_message(&payload, ROOM, self.store, self.ctx);
    }

    fn selected_user(&self) -> Option<String> {
        self.ctx.room_store.rooms_signal().get_untracked()[0]
            .selected_user
            .clone()
    }
}

#[test]
fn own_shoot_records_our_weapon() {
    let fixture = Fixture::new();

    fixture.handle(JyankenResponseMessage::OwnShoot {
        weapon: JyankenWeapon::Paper,
    });

    assert_eq!(
        fixture.store.state(ROOM).own_weapon,
        Some(JyankenWeapon::Paper)
    );
}

#[test]
fn game_status_updates_players_and_spectators() {
    let fixture = Fixture::new();

    fixture.handle(JyankenResponseMessage::GameStatus {
        selected: HashMap::from([("me".to_string(), true)]),
        spectators: vec!["u2".to_string()],
    });

    let state = fixture.store.state(ROOM);
    assert_eq!(state.selected, HashMap::from([("me".to_string(), true)]));
    assert!(state.spectators.contains("u2"));
    assert!(!state.spectators.contains("me"));
}

#[test]
fn game_results_are_stored() {
    let fixture = Fixture::new();
    let weapons = HashMap::from([("me".to_string(), JyankenWeapon::Rock)]);
    let scoreboard = HashMap::from([("me".to_string(), (0, 4))]);

    fixture.handle(JyankenResponseMessage::GameResults {
        weapons: weapons.clone(),
        scoreboard: scoreboard.clone(),
    });

    let state = fixture.store.state(ROOM);
    assert_eq!(state.results, Some(weapons));
    assert_eq!(state.scoreboard, scoreboard);
}

#[test]
fn new_game_resets_the_round_and_the_randomizer_pick() {
    let fixture = Fixture::new();
    fixture.handle(JyankenResponseMessage::GameResults {
        weapons: HashMap::new(),
        scoreboard: HashMap::new(),
    });

    fixture.handle(JyankenResponseMessage::NewGame);

    assert_eq!(fixture.store.state(ROOM).results, None);
    assert_eq!(fixture.selected_user(), None);
}

#[test]
fn game_over_is_shown_as_an_error() {
    let fixture = Fixture::new();

    fixture.handle(JyankenResponseMessage::GameOver);

    assert_eq!(
        fixture.ctx.last_error.get_untracked().as_deref(),
        Some("The game is already over.")
    );
}

#[test]
fn unreadable_payload_is_ignored() {
    let fixture = Fixture::new();
    let before = fixture.store.state(ROOM);

    handle_message("garbage", ROOM, fixture.store, fixture.ctx);

    assert_eq!(fixture.store.state(ROOM), before);
    assert_eq!(fixture.ctx.last_error.get_untracked(), None);
    assert_eq!(fixture.selected_user().as_deref(), Some("u2"));
}
