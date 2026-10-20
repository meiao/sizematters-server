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

use crate::stores::RoomStore;
use crate::ws;
use leptos::prelude::*;
use serde::Serialize;

/// The shareable URL of a room, `{origin}/room/{type}/{name}/{hashed password}`. Opening it
/// creates the room if it doesn't exist yet, or joins it.
pub(super) fn room_link(room_store: RoomStore, room_name: String) -> Memo<String> {
    Memo::new(move |_| {
        let origin = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_default();
        room_store.rooms_signal().with(|rooms| {
            rooms
                .iter()
                .find(|r| r.room_name == room_name)
                .map(|r| {
                    format!(
                        "{}/room/{}/{}/{}",
                        origin,
                        r.room_type.url_slug(),
                        room_name,
                        r.hashed_password
                    )
                })
                .unwrap_or_default()
        })
    })
}

/// Send a room-type-specific request (e.g. a `SizingRequestMessage`) to a room, as the
/// payload of a `RoomMessage`.
pub fn send_room_msg<T: Serialize>(room_name: &str, msg: &T) {
    match serde_json::to_string(msg) {
        Ok(payload) => ws::ws_room_msg(room_name.to_string(), payload),
        Err(err) => log::warn!("Failed to serialize message: {}", err),
    }
}
