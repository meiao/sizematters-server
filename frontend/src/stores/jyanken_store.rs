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

use leptos::prelude::*;
use sizematters_shared::messages::jyanken::JyankenWeapon;
use std::collections::{HashMap, HashSet};

/// Everything the client knows about one Jyanken room.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct JyankenState {
    /// user_id -> has shot. Only contains non-spectators.
    pub selected: HashMap<String, bool>,
    pub spectators: HashSet<String>,
    /// The weapon this client shot in the current round.
    pub own_weapon: Option<JyankenWeapon>,
    /// Set once everyone shot: user_id -> weapon.
    pub results: Option<HashMap<String, JyankenWeapon>>,
    /// user_id -> (wins last round, total wins).
    pub scoreboard: HashMap<String, (u32, u32)>,
}

/// Reactive store for Jyanken rooms, keyed by room name.
#[derive(Clone, Copy)]
pub struct JyankenStore {
    rooms: RwSignal<HashMap<String, JyankenState>>,
}

impl JyankenStore {
    pub fn new() -> Self {
        Self {
            rooms: RwSignal::new(HashMap::new()),
        }
    }

    fn update_room(&self, room_name: &str, f: impl FnOnce(&mut JyankenState)) {
        self.rooms
            .update(|rooms| f(rooms.entry(room_name.to_string()).or_default()));
    }

    pub fn state(&self, room_name: &str) -> JyankenState {
        self.rooms
            .with(|rooms| rooms.get(room_name).cloned().unwrap_or_default())
    }

    /// Everyone starts as a spectator server-side; the next GameStatus corrects it.
    pub fn room_joined(&self, room_name: &str, user_ids: &[String]) {
        self.rooms.update(|rooms| {
            rooms.insert(
                room_name.to_string(),
                JyankenState {
                    spectators: user_ids.iter().cloned().collect(),
                    ..Default::default()
                },
            );
        });
    }

    pub fn leave_room(&self, room_name: &str) {
        self.rooms.update(|rooms| {
            rooms.remove(room_name);
        });
    }

    pub fn user_joined(&self, room_name: &str, user_id: &str) {
        self.update_room(room_name, |s| {
            s.spectators.insert(user_id.to_string());
        });
    }

    pub fn user_left(&self, room_name: &str, user_id: &str) {
        self.update_room(room_name, |s| {
            s.spectators.remove(user_id);
            s.selected.remove(user_id);
        });
    }

    pub fn own_shoot(&self, room_name: &str, weapon: JyankenWeapon) {
        self.update_room(room_name, |s| s.own_weapon = Some(weapon));
    }

    pub fn game_status(
        &self,
        room_name: &str,
        selected: HashMap<String, bool>,
        spectators: Vec<String>,
    ) {
        self.update_room(room_name, |s| {
            s.selected = selected;
            s.spectators = spectators.into_iter().collect();
            // Being moved to the spectators drops your pick.
            if s.results.is_none() && !s.selected.values().any(|&shot| shot) {
                s.own_weapon = None;
            }
        });
    }

    pub fn game_results(
        &self,
        room_name: &str,
        weapons: HashMap<String, JyankenWeapon>,
        scoreboard: HashMap<String, (u32, u32)>,
    ) {
        self.update_room(room_name, |s| {
            s.results = Some(weapons);
            s.scoreboard = scoreboard;
        });
    }

    pub fn new_game(&self, room_name: &str) {
        self.update_room(room_name, |s| {
            s.results = None;
            s.own_weapon = None;
            s.selected.values_mut().for_each(|shot| *shot = false);
        });
    }
}
