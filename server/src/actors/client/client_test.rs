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

//! End-to-end tests: a real server with the `/ws` route, and websocket clients talking JSON
//! to it, like the frontend does.

use super::*;
use actix_web::{web, App};
use awc::ws::{Codec, Frame, Message as WsMessage};
use awc::BoxedSocket;
use futures_util::{SinkExt, StreamExt};
use sizematters_shared::messages::sizing::SizingResponseMessage;

/// How long to wait for a frame before failing the test.
const RECV_TIMEOUT: Duration = Duration::from_secs(5);

fn start_server() -> actix_test::TestServer {
    let room_manager = RoomManagerActor::new().start();
    actix_test::start(move || {
        App::new()
            .app_data(web::Data::new(room_manager.clone()))
            .service(web::resource("/ws").route(web::get().to(crate::ws_index)))
    })
}

struct WsClient {
    framed: actix_codec::Framed<BoxedSocket, Codec>,
}

impl WsClient {
    async fn connect(srv: &actix_test::TestServer) -> Self {
        WsClient {
            framed: awc::Client::new()
                .ws(srv.url("/ws"))
                .connect()
                .await
                .expect("websocket handshake")
                .1,
        }
    }

    /// Connects and registers; returns the client and its user data.
    async fn registered(srv: &actix_test::TestServer) -> (Self, UserData) {
        let mut client = Self::connect(srv).await;
        client.send(ClientRequestMessage::Register).await;
        let user = client.own_data().await;
        (client, user)
    }

    async fn send(&mut self, msg: ClientRequestMessage) {
        self.send_text(serde_json::to_string(&msg).unwrap()).await;
    }

    async fn send_text(&mut self, text: String) {
        self.framed
            .send(WsMessage::Text(text.into()))
            .await
            .unwrap();
    }

    async fn frame(&mut self) -> Frame {
        actix_rt::time::timeout(RECV_TIMEOUT, self.framed.next())
            .await
            .expect("timed out waiting for a frame")
            .expect("connection closed")
            .expect("protocol error")
    }

    /// The next server message, skipping heartbeats.
    async fn recv(&mut self) -> ClientResponseMessage {
        loop {
            match self.frame().await {
                Frame::Text(bytes) => {
                    return serde_json::from_slice(&bytes).expect("server sent invalid JSON")
                }
                Frame::Ping(_) | Frame::Pong(_) => continue,
                other => panic!("unexpected frame {other:?}"),
            }
        }
    }

    /// Skips messages until one matches `pick`.
    async fn recv_until<T>(&mut self, pick: impl Fn(&ClientResponseMessage) -> Option<T>) -> T {
        loop {
            let msg = self.recv().await;
            if let Some(found) = pick(&msg) {
                return found;
            }
        }
    }

    async fn own_data(&mut self) -> UserData {
        self.recv_until(|m| match m {
            ClientResponseMessage::OwnData { user } => Some(user.clone()),
            _ => None,
        })
        .await
    }

    async fn error(&mut self) -> String {
        self.recv_until(|m| match m {
            ClientResponseMessage::Error { msg } => Some(msg.clone()),
            _ => None,
        })
        .await
    }

    /// Waits for `RoomJoined` and returns the hashed password and room type.
    async fn room_joined(&mut self) -> (String, RoomType) {
        self.recv_until(|m| match m {
            ClientResponseMessage::RoomJoined {
                hashed_password,
                room_type,
                ..
            } => Some((hashed_password.clone(), *room_type)),
            _ => None,
        })
        .await
    }

    async fn create_room(&mut self, room_name: &str, password: &str, room_type: RoomType) {
        self.send(ClientRequestMessage::CreateRoom {
            room_name: room_name.to_string(),
            password: password.to_string(),
            password_is_hash: false,
            room_type,
        })
        .await;
    }
}

#[actix_rt::test]
async fn register_returns_a_fresh_identity() {
    let srv = start_server();

    let (_client, user) = WsClient::registered(&srv).await;

    assert_eq!(user.user_id.len(), 32);
    assert!(user.user_id.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(user.name, "Shirtless Muppet");
    assert_eq!(user.avatar_style, AvatarStyle::Identicon);
    assert_eq!(user.avatar, avatar::default_avatar(&user.user_id));
}

#[actix_rt::test]
async fn every_connection_gets_its_own_identity() {
    let srv = start_server();

    let (_a, first) = WsClient::registered(&srv).await;
    let (_b, second) = WsClient::registered(&srv).await;

    assert_ne!(first.user_id, second.user_id);
}

#[actix_rt::test]
async fn registering_again_keeps_the_identity() {
    let srv = start_server();
    let (mut client, first) = WsClient::registered(&srv).await;

    client.send(ClientRequestMessage::Register).await;

    assert_eq!(client.own_data().await, first);
}

#[actix_rt::test]
async fn set_name_updates_name_and_avatar() {
    let srv = start_server();
    let (mut client, before) = WsClient::registered(&srv).await;

    client
        .send(ClientRequestMessage::SetName {
            name: "Ann".to_string(),
        })
        .await;

    let after = client.own_data().await;
    assert_eq!(after.user_id, before.user_id);
    assert_eq!(after.name, "Ann");
    // The avatar is seeded with the name once the user picked one.
    assert_eq!(
        after.avatar,
        avatar::styled_avatar("Ann", &AvatarStyle::Identicon)
    );
}

#[actix_rt::test]
async fn set_avatar_style_updates_style_and_avatar() {
    let srv = start_server();
    let (mut client, _) = WsClient::registered(&srv).await;
    client
        .send(ClientRequestMessage::SetName {
            name: "Ann".to_string(),
        })
        .await;
    client.own_data().await;

    client
        .send(ClientRequestMessage::SetAvatarStyle {
            avatar_style: AvatarStyle::Pixelbot,
        })
        .await;

    let user = client.own_data().await;
    assert_eq!(user.name, "Ann");
    assert_eq!(user.avatar_style, AvatarStyle::Pixelbot);
    assert_eq!(
        user.avatar,
        avatar::styled_avatar("Ann", &AvatarStyle::Pixelbot)
    );
}

#[actix_rt::test]
async fn oversized_messages_are_dropped() {
    let srv = start_server();
    let mut client = WsClient::connect(&srv).await;

    client.send_text("x".repeat(1025)).await;

    assert_eq!(client.error().await, "Message dropped: max size exceeded.");
}

#[actix_rt::test]
async fn messages_up_to_1024_bytes_are_read() {
    let srv = start_server();
    let mut client = WsClient::connect(&srv).await;
    let name = "n".repeat(1024 - r#"{"type":"SetName","data":{"name":""}}"#.len());
    let text =
        serde_json::to_string(&ClientRequestMessage::SetName { name: name.clone() }).unwrap();
    assert_eq!(text.len(), 1024);

    client.send_text(text).await;

    assert_eq!(client.own_data().await.name, name);
}

#[actix_rt::test]
async fn unreadable_messages_are_echoed_back_as_errors() {
    let srv = start_server();
    let mut client = WsClient::connect(&srv).await;

    client.send_text(r#"{"type":"Dance"}"#.to_string()).await;

    assert_eq!(client.error().await, r#"{"type":"Dance"}"#);
}

#[actix_rt::test]
async fn connection_survives_bad_messages() {
    let srv = start_server();
    let mut client = WsClient::connect(&srv).await;
    client.send_text("not json".to_string()).await;
    client.error().await;

    client.send(ClientRequestMessage::Register).await;

    client.own_data().await;
}

#[actix_rt::test]
async fn binary_messages_are_ignored() {
    let srv = start_server();
    let mut client = WsClient::connect(&srv).await;

    client
        .framed
        .send(WsMessage::Binary(vec![1, 2, 3].into()))
        .await
        .unwrap();
    client.send(ClientRequestMessage::Register).await;

    // The first reply is to Register: nothing was sent for the binary frame.
    assert!(matches!(
        client.recv().await,
        ClientResponseMessage::OwnData { .. }
    ));
}

#[actix_rt::test]
async fn ping_is_answered_with_pong() {
    let srv = start_server();
    let mut client = WsClient::connect(&srv).await;

    client
        .framed
        .send(WsMessage::Ping("hello".into()))
        .await
        .unwrap();

    loop {
        match client.frame().await {
            Frame::Pong(bytes) => {
                assert_eq!(&bytes[..], b"hello");
                return;
            }
            Frame::Ping(_) => continue,
            other => panic!("unexpected frame {other:?}"),
        }
    }
}

#[actix_rt::test]
async fn create_room_joins_it() {
    let srv = start_server();
    let (mut client, _) = WsClient::registered(&srv).await;

    client.create_room("team", "secret", RoomType::Sizing).await;

    let (hashed_password, room_type) = client.room_joined().await;
    assert_eq!(hashed_password, "5ebe2294ecd0e0f08eab7690d2a6ee69");
    assert_eq!(room_type, RoomType::Sizing);
    let payload = client
        .recv_until(|m| match m {
            ClientResponseMessage::RoomMessage { payload, .. } => Some(payload.clone()),
            _ => None,
        })
        .await;
    assert!(matches!(
        serde_json::from_str(&payload),
        Ok(SizingResponseMessage::VoteStatus { .. })
    ));
}

#[actix_rt::test]
async fn invalid_room_name_is_reported() {
    let srv = start_server();
    let (mut client, _) = WsClient::registered(&srv).await;

    client
        .create_room("no spaces", "pw", RoomType::Sizing)
        .await;

    assert!(matches!(
        client.recv().await,
        ClientResponseMessage::InvalidRoomName
    ));
}

/// Two registered clients in room "team", created by the first one. Returns them with their ids.
async fn two_in_a_room(srv: &actix_test::TestServer) -> ((WsClient, String), (WsClient, String)) {
    let (mut owner, owner_data) = WsClient::registered(srv).await;
    let (mut guest, guest_data) = WsClient::registered(srv).await;
    owner.create_room("team", "secret", RoomType::Sizing).await;
    let (hashed_password, _) = owner.room_joined().await;

    // Join with the hashed password, as a shared room link does.
    guest
        .send(ClientRequestMessage::JoinRoom {
            room_name: "team".to_string(),
            password: hashed_password,
            password_is_hash: true,
        })
        .await;
    guest.room_joined().await;
    let guest_id = guest_data.user_id.clone();
    owner
        .recv_until(|m| match m {
            ClientResponseMessage::UserJoined { user, .. } if user.user_id == guest_id => Some(()),
            _ => None,
        })
        .await;
    ((owner, owner_data.user_id), (guest, guest_data.user_id))
}

#[actix_rt::test]
async fn wrong_password_is_reported() {
    let srv = start_server();
    let (mut owner, _) = WsClient::registered(&srv).await;
    let (mut guest, _) = WsClient::registered(&srv).await;
    owner.create_room("team", "secret", RoomType::Sizing).await;
    owner.room_joined().await;

    guest
        .send(ClientRequestMessage::JoinRoom {
            room_name: "team".to_string(),
            password: "guess".to_string(),
            password_is_hash: false,
        })
        .await;

    guest
        .recv_until(|m| match m {
            ClientResponseMessage::WrongPassword { room_name } => Some(room_name.clone()),
            _ => None,
        })
        .await;
}

#[actix_rt::test]
async fn room_messages_reach_the_room() {
    let srv = start_server();
    let ((mut owner, owner_id), (mut guest, _)) = two_in_a_room(&srv).await;

    owner
        .send(ClientRequestMessage::RoomMessage {
            room_name: "team".to_string(),
            payload: r#"{"type":"RegisterToVote"}"#.to_string(),
        })
        .await;

    // The guest sees the owner become a voter.
    guest
        .recv_until(|m| match m {
            ClientResponseMessage::RoomMessage { payload, .. } => {
                match serde_json::from_str(payload) {
                    Ok(SizingResponseMessage::VoteStatus { votes, .. })
                        if votes.contains_key(&owner_id) =>
                    {
                        Some(())
                    }
                    _ => None,
                }
            }
            _ => None,
        })
        .await;
}

#[actix_rt::test]
async fn name_changes_are_shown_to_room_members() {
    let srv = start_server();
    let ((mut owner, _), (mut guest, guest_id)) = two_in_a_room(&srv).await;

    guest
        .send(ClientRequestMessage::SetName {
            name: "Bob".to_string(),
        })
        .await;

    let updated = owner
        .recv_until(|m| match m {
            ClientResponseMessage::UserUpdated { user } => Some(user.clone()),
            _ => None,
        })
        .await;
    assert_eq!(updated.user_id, guest_id);
    assert_eq!(updated.name, "Bob");
}

#[actix_rt::test]
async fn leaving_a_room_is_shown_to_everyone() {
    let srv = start_server();
    let ((mut owner, _), (mut guest, guest_id)) = two_in_a_room(&srv).await;

    guest
        .send(ClientRequestMessage::LeaveRoom {
            room_name: "team".to_string(),
        })
        .await;

    for client in [&mut owner, &mut guest] {
        let left = client
            .recv_until(|m| match m {
                ClientResponseMessage::UserLeft { user_id, .. } => Some(user_id.clone()),
                _ => None,
            })
            .await;
        assert_eq!(left, guest_id);
    }
}

#[actix_rt::test]
async fn disconnecting_leaves_the_rooms() {
    let srv = start_server();
    let ((mut owner, _), (mut guest, guest_id)) = two_in_a_room(&srv).await;

    guest.framed.send(WsMessage::Close(None)).await.unwrap();

    let left = owner
        .recv_until(|m| match m {
            ClientResponseMessage::UserLeft { user_id, .. } => Some(user_id.clone()),
            _ => None,
        })
        .await;
    assert_eq!(left, guest_id);
}

#[actix_rt::test]
async fn dropping_the_connection_leaves_the_rooms() {
    let srv = start_server();
    let ((mut owner, _), (guest, guest_id)) = two_in_a_room(&srv).await;

    drop(guest);

    let left = owner
        .recv_until(|m| match m {
            ClientResponseMessage::UserLeft { user_id, .. } => Some(user_id.clone()),
            _ => None,
        })
        .await;
    assert_eq!(left, guest_id);
}
