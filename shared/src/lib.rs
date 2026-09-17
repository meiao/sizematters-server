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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct UserData {
    pub user_id: String,
    pub name: String,
    pub gravatar_id: String,
}

impl UserData {
    pub fn new(user_id: String) -> UserData {
        let name = "Shirtless Muppet".to_string();
        let gravatar_id = format!("{:x}", md5::compute(user_id.clone()));
        UserData {
            user_id,
            name,
            gravatar_id,
        }
    }

    pub fn set_avatar(&mut self, avatar: &str) {
        self.gravatar_id = format!("{:x}", md5::compute(avatar));
    }
}

/// Messages sent from the client to the server.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "actix", derive(actix::Message))]
#[cfg_attr(feature = "actix", rtype(result = "()"))]
#[serde(tag = "type", content = "data")]
pub enum ClientRequestMessage {
    Register,
    SetName {
        name: String,
    },
    SetAvatar {
        avatar: String,
    },
    JoinRoom {
        room_name: String,
        password: String,
        password_is_hash: bool,
    },
    LeaveRoom {
        room_name: String,
    },
    Vote {
        room_name: String,
        size: u64,
    },
    NewVote {
        room_name: String,
    },
    Randomize {
        room_name: String,
    },
}

/// Messages sent from the server to the client.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "actix", derive(actix::Message))]
#[cfg_attr(feature = "actix", rtype(result = "()"))]
#[serde(tag = "type", content = "data")]
pub enum ClientResponseMessage {
    RoomJoined {
        room_name: String,
        hashed_password: String,
        users: Vec<UserData>,
        votes_cast: usize,
    },
    UserJoined {
        room_name: String,
        user: UserData,
    },
    UserLeft {
        room_name: String,
        user_id: String,
    },
    UserUpdated {
        user: UserData,
    },
    OwnData {
        user: UserData,
    },
    OwnVote {
        room_name: String,
        size: u64,
    },
    VoteStatus {
        room_name: String,
        votes: HashMap<String, bool>,
    },
    VoteResults {
        room_name: String,
        votes: HashMap<String, u64>,
    },
    NewVote {
        room_name: String,
    },
    AlreadyInRoom {
        room_name: String,
    },
    WrongPassword {
        room_name: String,
    },
    Randomized {
        room_name: String,
        selected_user_id: String,
    },
    InvalidRoomName,
    VotingOver,
    CannotJoinMultipleRooms,
    Error {
        msg: String,
    },
}
