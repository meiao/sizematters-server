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

//! Helpers shared by the room tests: an actor that records every message it receives, so it can
//! stand in for a `ClientActor` or the `RoomManagerActor`, and shortcuts to drive a room.

use crate::actors::messages::RoomMessages;
use crate::actors::rooms::room::{Room, RoomBehavior};
use actix::prelude::*;
use serde::de::DeserializeOwned;
use sizematters_shared::avatar::AvatarStyle;
use sizematters_shared::messages::ClientResponseMessage;
use sizematters_shared::{RoomType, UserData};
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;

/// Records every `M` it receives until they are drained.
pub(super) struct Collector<M> {
    received: Vec<M>,
}

impl<M: Unpin + 'static> Actor for Collector<M> {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // The rooms use `try_send`, which fails on a full mailbox (16 by default).
        ctx.set_mailbox_capacity(1024);
    }
}

impl<M> Handler<M> for Collector<M>
where
    M: Message<Result = ()> + Unpin + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: M, _ctx: &mut Self::Context) {
        self.received.push(msg);
    }
}

/// Returns (and forgets) everything received so far.
struct Drain<M>(PhantomData<M>);

impl<M: 'static> Message for Drain<M> {
    type Result = Vec<M>;
}

impl<M: Unpin + 'static> Handler<Drain<M>> for Collector<M> {
    type Result = MessageResult<Drain<M>>;

    fn handle(&mut self, _msg: Drain<M>, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(std::mem::take(&mut self.received))
    }
}

/// Stops the collector, so sending to it fails like sending to a disconnected client.
/// Generic over `M` only so that it can't overlap with the `Handler<M>` impl.
struct Stop<M>(PhantomData<M>);

impl<M: 'static> Message for Stop<M> {
    type Result = ();
}

impl<M: Unpin + 'static> Handler<Stop<M>> for Collector<M> {
    type Result = ();

    fn handle(&mut self, _msg: Stop<M>, ctx: &mut Self::Context) {
        ctx.stop();
    }
}

/// A recording stand-in for an actor receiving `M`.
pub(super) struct Probe<M: Unpin + 'static> {
    addr: Addr<Collector<M>>,
}

impl<M> Probe<M>
where
    M: Message<Result = ()> + Send + Unpin + 'static,
{
    pub(super) fn start() -> Self {
        Probe {
            addr: Collector {
                received: Vec::new(),
            }
            .start(),
        }
    }

    pub(super) fn recipient(&self) -> Recipient<M> {
        self.addr.clone().recipient()
    }

    /// Everything received since the last call. Messages sent before this call are included,
    /// since they are ahead of the drain request in the mailbox.
    pub(super) async fn take(&self) -> Vec<M> {
        self.addr.send(Drain(PhantomData)).await.unwrap()
    }

    /// For messages that go through another actor first (e.g. manager -> room -> client):
    /// keeps collecting until `done` holds for what was received, or panics after a timeout.
    pub(super) async fn take_until(&self, done: impl Fn(&[M]) -> bool) -> Vec<M> {
        let mut received = Vec::new();
        for _ in 0..200 {
            received.extend(self.take().await);
            if done(&received) {
                return received;
            }
            actix_rt::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("timed out waiting for messages");
    }

    /// Waits a little, then returns what arrived. Used to assert that nothing was sent.
    pub(super) async fn take_after_settling(&self) -> Vec<M> {
        actix_rt::time::sleep(Duration::from_millis(50)).await;
        self.take().await
    }

    pub(super) async fn stop(&self) {
        self.addr.send(Stop(PhantomData)).await.unwrap();
        while self.addr.connected() {
            actix_rt::task::yield_now().await;
        }
    }
}

pub(super) type ClientProbe = Probe<ClientResponseMessage>;
pub(super) type ManagerProbe = Probe<RoomMessages>;

pub(super) fn user(user_id: &str) -> UserData {
    UserData {
        user_id: user_id.to_string(),
        name: format!("name of {user_id}"),
        avatar: format!("avatar of {user_id}"),
        avatar_style: AvatarStyle::default(),
    }
}

pub(super) fn arc(s: &str) -> Arc<String> {
    Arc::new(s.to_string())
}

pub(super) const ROOM: &str = "room-1";
pub(super) const PASSWORD: &str = "secret";

/// Starts a room named `ROOM`, with plaintext password `PASSWORD`, reporting to `manager`.
pub(super) fn start_room<T: DeserializeOwned + 'static>(
    manager: &ManagerProbe,
    room_type: RoomType,
    behavior: Box<dyn RoomBehavior<T>>,
) -> Addr<Room<T>> {
    Room::new(
        arc(ROOM),
        arc(PASSWORD),
        false,
        manager.recipient(),
        room_type,
        behavior,
    )
    .start()
}

pub(super) fn join_msg(user_id: &str, password: &str, client: &ClientProbe) -> RoomMessages {
    RoomMessages::JoinRoom {
        room_name: arc(ROOM),
        password: arc(password),
        password_is_hash: false,
        user: user(user_id),
        recipient: client.recipient(),
    }
}

pub(super) fn specific_msg(user_id: &str, payload: &str) -> RoomMessages {
    RoomMessages::SpecificMessage {
        room_name: arc(ROOM),
        user_id: arc(user_id),
        payload: payload.to_string(),
    }
}

/// The payloads of the `RoomMessage`s among `msgs`, decoded as `T`.
pub(super) fn room_payloads<T: DeserializeOwned>(msgs: &[ClientResponseMessage]) -> Vec<T> {
    msgs.iter()
        .filter_map(|msg| match msg {
            ClientResponseMessage::RoomMessage { room_name, payload } => {
                assert_eq!(room_name, ROOM);
                Some(serde_json::from_str(payload).expect("room payload should parse"))
            }
            _ => None,
        })
        .collect()
}

/// The texts of the `Error` messages among `msgs`.
pub(super) fn errors(msgs: &[ClientResponseMessage]) -> Vec<String> {
    msgs.iter()
        .filter_map(|msg| match msg {
            ClientResponseMessage::Error { msg } => Some(msg.clone()),
            _ => None,
        })
        .collect()
}

/// A running room plus a probe for every user that joined it.
pub(super) struct RoomFixture<T: 'static> {
    _manager: ManagerProbe,
    pub(super) room: Addr<Room<T>>,
    clients: Vec<(String, ClientProbe)>,
}

impl<T: DeserializeOwned + 'static> RoomFixture<T> {
    pub(super) fn new(room_type: RoomType, behavior: Box<dyn RoomBehavior<T>>) -> Self {
        let manager = ManagerProbe::start();
        let room = start_room(&manager, room_type, behavior);
        RoomFixture {
            _manager: manager,
            room,
            clients: Vec::new(),
        }
    }

    pub(super) async fn join(&mut self, user_id: &str) {
        let client = ClientProbe::start();
        self.room
            .send(join_msg(user_id, PASSWORD, &client))
            .await
            .unwrap();
        self.clients.push((user_id.to_string(), client));
    }

    /// Sends a room specific message (JSON `payload`) on behalf of `user_id`.
    pub(super) async fn send(&self, user_id: &str, payload: &str) {
        self.room
            .send(specific_msg(user_id, payload))
            .await
            .unwrap();
    }

    pub(super) async fn leave(&self, user_id: &str) {
        self.room
            .send(RoomMessages::UserLeft {
                user_id: arc(user_id),
            })
            .await
            .unwrap();
    }

    fn client(&self, user_id: &str) -> &ClientProbe {
        &self
            .clients
            .iter()
            .find(|(id, _)| id == user_id)
            .expect("unknown client")
            .1
    }

    /// Everything `user_id` received since the last call.
    pub(super) async fn take(&self, user_id: &str) -> Vec<ClientResponseMessage> {
        self.client(user_id).take().await
    }

    /// The room specific messages `user_id` received since the last call.
    pub(super) async fn payloads<R: DeserializeOwned>(&self, user_id: &str) -> Vec<R> {
        room_payloads(&self.take(user_id).await)
    }

    pub(super) async fn drain_all(&self) {
        for (_, client) in &self.clients {
            client.take().await;
        }
    }
}
