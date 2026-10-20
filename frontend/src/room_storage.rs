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

//! Persistence of the rooms the user is in, so they can be restored on the next page load.
//!
//! localStorage layout:
//! - `rooms`: JSON array of room names, without duplicates.
//! - `room.{roomName}`: JSON object with the data needed to recreate that room.

use serde::{Deserialize, Serialize};
use sizematters_shared::RoomType;

const ROOMS_KEY: &str = "rooms";

/// Everything needed to recreate (or rejoin) a room, saved under `room.{roomName}`.
#[derive(Serialize, Deserialize)]
struct SavedRoom {
    hashed_password: String,
    room_type: RoomType,
}

/// A room read back from localStorage.
pub struct StoredRoom {
    pub room_name: String,
    pub hashed_password: String,
    pub room_type: RoomType,
}

fn room_key(room_name: &str) -> String {
    format!("room.{room_name}")
}

/// The page's localStorage, or `None` when it is unavailable (e.g. blocked by privacy
/// settings or a sandboxed iframe, where `window.localStorage` throws).
pub(crate) fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn read_room(storage: &web_sys::Storage, room_name: &str) -> Option<SavedRoom> {
    storage
        .get_item(&room_key(room_name))
        .ok()
        .flatten()
        .and_then(|json| serde_json::from_str(&json).ok())
}

fn saved_room_names(storage: &web_sys::Storage) -> Vec<String> {
    storage
        .get_item(ROOMS_KEY)
        .ok()
        .flatten()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn write_room_names(storage: &web_sys::Storage, names: &[String]) {
    if let Ok(json) = serde_json::to_string(names) {
        let _ = storage.set_item(ROOMS_KEY, &json);
    }
}

/// Save a joined room.
pub fn save_room(room_name: &str, hashed_password: &str, room_type: RoomType) {
    let Some(storage) = local_storage() else {
        return;
    };
    let saved = SavedRoom {
        hashed_password: hashed_password.to_string(),
        room_type,
    };
    if let Ok(json) = serde_json::to_string(&saved) {
        let _ = storage.set_item(&room_key(room_name), &json);
    }
    let mut names = saved_room_names(&storage);
    if !names.iter().any(|n| n == room_name) {
        names.push(room_name.to_string());
        write_room_names(&storage, &names);
    }
}

/// Remove a room the user left.
pub fn forget_room(room_name: &str) {
    let Some(storage) = local_storage() else {
        return;
    };
    let mut names = saved_room_names(&storage);
    names.retain(|n| n != room_name);
    write_room_names(&storage, &names);
    let _ = storage.remove_item(&room_key(room_name));
}

/// All saved rooms. Entries whose data is missing or unreadable are dropped.
pub fn saved_rooms() -> Vec<StoredRoom> {
    let Some(storage) = local_storage() else {
        return Vec::new();
    };
    let mut rooms = Vec::new();
    for room_name in saved_room_names(&storage) {
        match read_room(&storage, &room_name) {
            Some(saved) => rooms.push(StoredRoom {
                room_name,
                hashed_password: saved.hashed_password,
                room_type: saved.room_type,
            }),
            None => forget_room(&room_name),
        }
    }
    rooms
}
