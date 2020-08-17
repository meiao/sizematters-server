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
pub enum RoomMessage {
    JoinRoom {
        room_name: String,
        password: String,
        password_is_hash: bool,
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
    NewVote {
        room_name: String,
        user_id: String,
    },
    UserUpdated {
        user: UserData,
    },
    UserLeft {
        user_id: String,
    },
    RoomClosing {
        room_name: String,
    },
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
