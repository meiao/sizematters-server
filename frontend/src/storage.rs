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

use web_sys::Storage;

fn local_storage() -> Option<Storage> {
    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
}

// User profile

pub fn save_name(name: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item("name", name);
    }
}

pub fn load_name() -> Option<String> {
    local_storage().and_then(|s| s.get_item("name").ok().flatten())
}

pub fn save_avatar(email: &str) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item("avatar", email);
    }
}

pub fn load_avatar() -> Option<String> {
    local_storage().and_then(|s| s.get_item("avatar").ok().flatten())
}

// Recent rooms

const RECENT_ROOMS_KEY: &str = "recent_rooms";
const MAX_RECENT_ROOMS: usize = 10;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RecentRoom {
    pub room_name: String,
    pub hashed_password: String,
}

pub fn save_recent_room(room_name: &str, hashed_password: &str) {
    let storage = match local_storage() {
        Some(s) => s,
        None => return,
    };

    let mut rooms = load_recent_rooms();

    // Remove if already exists (will re-add at front)
    rooms.retain(|r| r.room_name != room_name);
    rooms.insert(0, RecentRoom {
        room_name: room_name.to_string(),
        hashed_password: hashed_password.to_string(),
    });
    rooms.truncate(MAX_RECENT_ROOMS);

    if let Ok(json) = serde_json::to_string(&rooms) {
        let _ = storage.set_item(RECENT_ROOMS_KEY, &json);
    }
}

pub fn remove_recent_room(room_name: &str) {
    let storage = match local_storage() {
        Some(s) => s,
        None => return,
    };

    let mut rooms = load_recent_rooms();
    rooms.retain(|r| r.room_name != room_name);

    if let Ok(json) = serde_json::to_string(&rooms) {
        let _ = storage.set_item(RECENT_ROOMS_KEY, &json);
    }
}

pub fn load_recent_rooms() -> Vec<RecentRoom> {
    local_storage()
        .and_then(|s| s.get_item(RECENT_ROOMS_KEY).ok().flatten())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}
