/*
 * SizeMatters - a ticket sizing util
 * Copyright (C) 2020 Andre Onuki
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

use crate::data::UserData;
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::collections::HashMap;

/// Messages sent from the client to the server.
#[derive(Message, Deserialize)]
#[serde(tag = "type", content = "data")]
#[rtype(result = "()")]
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
}

/// messages sent to a RoomActor
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct JoinRoom {
    pub room_name: String,
    pub password: String,
    pub password_is_hash: bool,
    pub user: UserData,
    pub recipient: Recipient<ClientResponseMessage>,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct LeaveRoom {
    pub room_name: String,
    pub user_id: String,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Vote {
    pub room_name: String,
    pub user_id: String,
    pub size: u64,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct NewVote {
    pub room_name: String,
    pub user_id: String,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct UserUpdated {
    pub user: UserData,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct UserLeft {
    pub user_id: String,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct RoomClosing {
    pub room_name: String,
}

/// Messages sent to the client
#[derive(Message, Serialize, Clone)]
#[serde(tag = "type", content = "data")]
#[rtype(result = "()")]
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
    VotesCast {
        room_name: String,
        votes_cast: usize,
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
    InvalidRoomName,
    VotingOver,
    Error {
        msg: String,
    },
}
