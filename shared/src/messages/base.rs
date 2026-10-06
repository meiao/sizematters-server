use crate::{RoomType, UserData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    CreateRoom {
        room_name: String,
        password: String,
        password_is_hash: bool,
        room_type: RoomType,
    },
    JoinRoom {
        room_name: String,
        password: String,
        password_is_hash: bool,
    },
    LeaveRoom {
        room_name: String,
    },
    RoomMessage {
        room_name: String,
        payload: String,
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
        size: String,
    },
    VoteStatus {
        room_name: String,
        votes: HashMap<String, bool>,
        spectators: Vec<String>,
    },
    VoteResults {
        room_name: String,
        votes: HashMap<String, String>,
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
