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
use sizematters_shared::{RoomType, UserData};

#[derive(Clone, Debug, PartialEq)]
pub struct RoomStatus {
    pub room_name: String,
    pub hashed_password: String,
    pub room_type: RoomType,
    pub users: Vec<UserData>,
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

    /// Record a joined room. If the room is already known (e.g. it was rejoined after a
    /// reconnect), its data is replaced with the server's, keeping its position in the list.
    pub fn room_joined(
        &self,
        room_name: String,
        hashed_password: String,
        room_type: RoomType,
        users: Vec<UserData>,
    ) {
        self.rooms.update(|rooms| {
            let status = RoomStatus {
                room_name,
                hashed_password,
                room_type,
                users,
                selected_user: None,
            };
            match rooms.iter_mut().find(|r| r.room_name == status.room_name) {
                Some(existing) => *existing = status,
                None => rooms.push(status),
            }
        });
    }

    /// Forget every room. Used when the connection is re-established, since the server
    /// gives us a new user id and the rooms get rejoined from localStorage.
    pub fn clear(&self) {
        self.rooms.update(|rooms| rooms.clear());
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

    /// Replace the stored copy of an updated user in every room that has them.
    pub fn user_updated(&self, user: &UserData) {
        self.rooms.update(|rooms| {
            for u in rooms
                .iter_mut()
                .flat_map(|r| r.users.iter_mut())
                .filter(|u| u.user_id == user.user_id)
            {
                *u = user.clone();
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

    /// The type of a joined room, without tracking.
    pub fn room_type_untracked(&self, room_name: &str) -> Option<RoomType> {
        self.rooms.with_untracked(|rooms| {
            rooms
                .iter()
                .find(|r| r.room_name == room_name)
                .map(|r| r.room_type)
        })
    }

    /// Forget who the randomizer picked, when a new round starts.
    pub fn clear_selected_user(&self, room_name: &str) {
        self.rooms.update(|rooms| {
            if let Some(room) = rooms.iter_mut().find(|r| r.room_name == room_name) {
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
