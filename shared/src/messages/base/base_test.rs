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

//! These messages are the wire protocol between the frontend and the server, so these tests
//! pin down their JSON shape (`{"type": ..., "data": ...}`).

use super::*;
use serde_json::{json, Value};

fn to_value<T: Serialize>(msg: &T) -> Value {
    serde_json::to_value(msg).unwrap()
}

fn user() -> UserData {
    UserData {
        user_id: "u1".to_string(),
        name: "Ann".to_string(),
        avatar: "data:x".to_string(),
        avatar_style: AvatarStyle::Glass,
    }
}

#[test]
fn unit_request_has_only_a_type() {
    assert_eq!(
        to_value(&ClientRequestMessage::Register),
        json!({"type": "Register"})
    );
}

#[test]
fn request_fields_go_under_data() {
    let msg = ClientRequestMessage::CreateRoom {
        room_name: "team".to_string(),
        password: "pw".to_string(),
        password_is_hash: false,
        room_type: RoomType::Jyanken,
    };
    assert_eq!(
        to_value(&msg),
        json!({
            "type": "CreateRoom",
            "data": {
                "room_name": "team",
                "password": "pw",
                "password_is_hash": false,
                "room_type": "Jyanken"
            }
        })
    );
}

#[test]
fn parses_requests_as_sent_by_the_frontend() {
    let msg: ClientRequestMessage =
        serde_json::from_str(r#"{"type":"SetName","data":{"name":"Bob"}}"#).unwrap();
    assert!(matches!(msg, ClientRequestMessage::SetName { name } if name == "Bob"));

    let msg: ClientRequestMessage =
        serde_json::from_str(r#"{"type":"SetAvatarStyle","data":{"avatar_style":"Shapes"}}"#)
            .unwrap();
    assert!(matches!(
        msg,
        ClientRequestMessage::SetAvatarStyle {
            avatar_style: AvatarStyle::Shapes
        }
    ));

    let msg: ClientRequestMessage = serde_json::from_str(
        r#"{"type":"RoomMessage","data":{"room_name":"r","payload":"{\"type\":\"NewVote\"}"}}"#,
    )
    .unwrap();
    assert!(matches!(
        msg,
        ClientRequestMessage::RoomMessage { room_name, payload }
            if room_name == "r" && payload == r#"{"type":"NewVote"}"#
    ));
}

#[test]
fn rejects_malformed_requests() {
    // unknown type
    assert!(serde_json::from_str::<ClientRequestMessage>(r#"{"type":"Nope"}"#).is_err());
    // missing field
    assert!(serde_json::from_str::<ClientRequestMessage>(
        r#"{"type":"JoinRoom","data":{"room_name":"r"}}"#
    )
    .is_err());
    // wrong field type
    assert!(serde_json::from_str::<ClientRequestMessage>(
        r#"{"type":"JoinRoom","data":{"room_name":"r","password":"p","password_is_hash":"yes"}}"#
    )
    .is_err());
    // not json
    assert!(serde_json::from_str::<ClientRequestMessage>("hello").is_err());
}

#[test]
fn unit_response_has_only_a_type() {
    assert_eq!(
        to_value(&ClientResponseMessage::InvalidRoomName),
        json!({"type": "InvalidRoomName"})
    );
}

#[test]
fn room_joined_response_shape() {
    let msg = ClientResponseMessage::RoomJoined {
        room_name: "team".to_string(),
        hashed_password: "abc".to_string(),
        room_type: RoomType::Sizing,
        users: vec![user()],
    };
    assert_eq!(
        to_value(&msg),
        json!({
            "type": "RoomJoined",
            "data": {
                "room_name": "team",
                "hashed_password": "abc",
                "room_type": "Sizing",
                "users": [{
                    "user_id": "u1",
                    "name": "Ann",
                    "avatar": "data:x",
                    "avatar_style": "Glass"
                }]
            }
        })
    );
}

#[test]
fn every_response_round_trips_through_json() {
    let room_name = || "r".to_string();
    let messages = vec![
        ClientResponseMessage::RoomJoined {
            room_name: room_name(),
            hashed_password: "h".to_string(),
            room_type: RoomType::Jyanken,
            users: vec![user()],
        },
        ClientResponseMessage::UserJoined {
            room_name: room_name(),
            user: user(),
        },
        ClientResponseMessage::UserLeft {
            room_name: room_name(),
            user_id: "u1".to_string(),
        },
        ClientResponseMessage::UserUpdated { user: user() },
        ClientResponseMessage::OwnData { user: user() },
        ClientResponseMessage::AlreadyInRoom {
            room_name: room_name(),
        },
        ClientResponseMessage::WrongPassword {
            room_name: room_name(),
        },
        ClientResponseMessage::Randomized {
            room_name: room_name(),
            selected_user_id: "u1".to_string(),
        },
        ClientResponseMessage::InvalidRoomName,
        ClientResponseMessage::RoomMessage {
            room_name: room_name(),
            payload: "{}".to_string(),
        },
        ClientResponseMessage::Error {
            msg: "boom".to_string(),
        },
    ];
    for msg in messages {
        let json = serde_json::to_string(&msg).unwrap();
        let back: ClientResponseMessage = serde_json::from_str(&json).unwrap();
        // The enum has no PartialEq; comparing the JSON is equivalent.
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}
