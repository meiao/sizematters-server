/*
 * SizeMatters - a ticket sizing util
 * Copyright (C) 2025 Andre Onuki
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

pub use sizematters_shared::UserData;

use actix::prelude::*;
pub use sizematters_shared::messages::ClientResponseMessage;
use std::clone::Clone;
use std::sync::Arc;

/// messages sent to a RoomActor
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub enum RoomMessages {
    JoinRoom {
        room_name: Arc<String>,
        password: Arc<String>,
        password_is_hash: bool,
        user: UserData,
        recipient: Recipient<ClientResponseMessage>,
    },
    LeaveRoom {
        room_name: Arc<String>,
        user_id: Arc<String>,
    },
    UserUpdated {
        user: UserData,
    },
    UserLeft {
        user_id: Arc<String>,
    },
    RoomClosing {
        room_name: Arc<String>,
    },
    SpecificMessage {
        room_name: Arc<String>,
        user_id: Arc<String>,
        payload: String,
    },
}
