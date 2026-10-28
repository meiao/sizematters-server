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

use super::*;
use crate::actors::rooms::test_support::*;
use sizematters_shared::messages::jyanken::JyankenResponseMessage;
use sizematters_shared::messages::sizing::SizingResponseMessage;
use sizematters_shared::messages::ClientResponseMessage;

/// Creates a room of `room_type`, joins it, and returns what the joining client received.
async fn create_and_join(
    room_type: RoomType,
    password: &str,
    is_hash: bool,
) -> Vec<ClientResponseMessage> {
    let manager = ManagerProbe::start();
    let client = ClientProbe::start();
    let room = create(
        arc(ROOM),
        arc(password),
        is_hash,
        room_type,
        manager.recipient(),
    );
    room.send(RoomMessages::JoinRoom {
        room_name: arc(ROOM),
        password: arc(password),
        password_is_hash: is_hash,
        user: user("u1"),
        recipient: client.recipient(),
    })
    .await
    .unwrap();
    client.take().await
}

fn joined_room_type(msgs: &[ClientResponseMessage]) -> RoomType {
    msgs.iter()
        .find_map(|m| match m {
            ClientResponseMessage::RoomJoined { room_type, .. } => Some(*room_type),
            _ => None,
        })
        .expect("expected RoomJoined")
}

#[actix_rt::test]
async fn creates_a_sizing_room() {
    let msgs = create_and_join(RoomType::Sizing, PASSWORD, false).await;

    assert_eq!(joined_room_type(&msgs), RoomType::Sizing);
    let payloads: Vec<SizingResponseMessage> = room_payloads(&msgs);
    assert!(matches!(
        payloads[..],
        [SizingResponseMessage::VoteStatus { .. }]
    ));
}

#[actix_rt::test]
async fn creates_a_jyanken_room() {
    let msgs = create_and_join(RoomType::Jyanken, PASSWORD, false).await;

    assert_eq!(joined_room_type(&msgs), RoomType::Jyanken);
    let payloads: Vec<JyankenResponseMessage> = room_payloads(&msgs);
    assert!(matches!(
        payloads[..],
        [JyankenResponseMessage::GameStatus { .. }]
    ));
}

#[actix_rt::test]
async fn passes_the_password_through() {
    let hash = "0123456789abcdef0123456789abcdef";
    let msgs = create_and_join(RoomType::Sizing, hash, true).await;

    assert!(msgs.iter().any(|m| matches!(
        m,
        ClientResponseMessage::RoomJoined { hashed_password, .. } if hashed_password == hash
    )));
}
