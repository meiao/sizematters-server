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
use sizematters_shared::UserData;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct RoomStatus {
    pub room_name: String,
    pub hashed_password: String,
    pub users: Vec<UserData>,
    pub votes_cast: usize,
    /// The user selected by the randomizer, if any.
    pub selected_user: Option<String>,
}

/// Reactive store for all rooms the user has joined.
#[derive(Clone, Copy)]
pub struct RoomStore {
    rooms: RwSignal<Vec<RoomStatus>>,
}

impl RoomStore {
    pub fn new() -> Self {
        Self {
            rooms: RwSignal::new(Vec::new()),
        }
    }

    pub fn rooms_signal(&self) -> RwSignal<Vec<RoomStatus>> {
        self.rooms
    }

    pub fn room_joined(
        &self,
        room_name: String,
        hashed_password: String,
        users: Vec<UserData>,
        votes_cast: usize,
    ) {
        self.rooms.update(|rooms| {
            if !rooms.iter().any(|r| r.room_name == room_name) {
                rooms.push(RoomStatus {
                    room_name,
                    hashed_password,
                    users,
                    votes_cast,
                    selected_user: None,
                });
            }
        });
    }

    pub fn user_joined(&self, room_name: &str, user: UserData) {
        self.rooms.update(|rooms| {
            if let Some(room) = rooms.iter_mut().find(|r| r.room_name == room_name) {
                if !room.users.iter().any(|u| u.user_id == user.user_id) {
                    room.users.push(user);
                }
            }
        });
    }

    pub fn user_left(&self, room_name: &str, user_id: &str) {
        self.rooms.update(|rooms| {
            if let Some(room) = rooms.iter_mut().find(|r| r.room_name == room_name) {
                room.users.retain(|u| u.user_id != user_id);
            }
        });
    }

    pub fn vote_status(&self, room_name: &str, votes: &HashMap<String, bool>) {
        self.rooms.update(|rooms| {
            if let Some(room) = rooms.iter_mut().find(|r| r.room_name == room_name) {
                room.votes_cast = votes.values().filter(|&&v| v).count();
            }
        });
    }

    /// Reset room state for a new voting round.
    pub fn new_vote(&self, room_name: &str) {
        self.rooms.update(|rooms| {
            if let Some(room) = rooms.iter_mut().find(|r| r.room_name == room_name) {
                room.votes_cast = 0;
                room.selected_user = None;
            }
        });
    }

    pub fn randomized(&self, room_name: &str, selected_user_id: &str) {
        self.rooms.update(|rooms| {
            if let Some(room) = rooms.iter_mut().find(|r| r.room_name == room_name) {
                room.selected_user = Some(selected_user_id.to_string());
            }
        });
    }

    pub fn leave_room(&self, room_name: &str) {
        self.rooms.update(|rooms| {
            rooms.retain(|r| r.room_name != room_name);
        });
    }
}
