use crate::data::UserData;
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Messages sent from the client to the server.
#[derive(Message, Deserialize)]
#[serde(tag = "type", content = "data")]
#[rtype(result = "()")]
pub enum ClientRequestMessage {
    Register,
    SetName { name: String },
    SetAvatar { avatar: String },
    JoinRoom { room_name: String, password: String },
    LeaveRoom { room_name: String },
    Vote { room_name: String, size: u64 },
}

/// messages sent to a RoomActor
#[derive(Message)]
#[rtype(result = "()")]
pub enum RoomMessage {
    JoinRoom {
        room_name: String,
        password: String,
        user: UserData,
        recipient: Recipient<ClientResponseMessage>,
    },
    LeaveRoom {
        room_name: String,
        user_id: String,
    },
    Vote {
        room_name: String,
        user_id: String,
        size: u64,
    },
    UserUpdated {
        user: UserData,
    },
}

/// Messages sent to the client
#[derive(Message, Serialize)]
#[serde(tag = "type", content = "data")]
#[rtype(result = "()")]
pub enum ClientResponseMessage {
    UserEntered {
        room_name: String,
        user: UserData,
    },
    UserLeft {
        room_name: String,
        user_id: String,
    },
    VoteResults {
        room_name: String,
        votes: HashMap<String, u64>,
    },
    NewVote {
        room_name: String,
    },
    UsersInRoom {
        room_name: String,
        users: Vec<UserData>,
    },
    AlreadyInRoom {
        room_name: String,
    },
    WrongPassword {
        room_name: String,
    },
    UserUpdated {
        user: UserData,
    },
    YourData {
        user: UserData,
    },
    Error {
        msg: String,
    },
}
