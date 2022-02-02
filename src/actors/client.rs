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

use actix::prelude::*;
use actix_web_actors::ws;
use serde_json::Error;
use std::time::Duration;
use std::time::Instant;
use uuid::Uuid;

use super::RoomManagerActor;
use crate::actors::messages::{ClientRequestMessage, ClientResponseMessage, RoomMessage};
use crate::data::UserData;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// websocket connection is long running connection, it easier
/// to handle with an actor
pub struct ClientActor {
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    last_heartbeat: Instant,
    user: UserData,
    room_manager: Addr<RoomManagerActor>,
}

impl ClientActor {
    pub fn new(room_manager: Addr<RoomManagerActor>) -> Self {
        let user_id = Uuid::new_v4().simple().to_string();
        Self {
            last_heartbeat: Instant::now(),
            user: UserData::new(user_id),
            room_manager,
        }
    }
}

impl Actor for ClientActor {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
    }
}

/// Handler for `ws::Message`
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ClientActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        // process websocket messages
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => self.text(text, ctx),
            Ok(ws::Message::Binary(_bin)) => {} // ignore binary
            Ok(ws::Message::Close(reason)) => {
                self.user_left();
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl ClientActor {
    fn text(&mut self, msg: String, ctx: &mut <Self as Actor>::Context) {
        let client_msg: Result<ClientRequestMessage, Error> = serde_json::from_str(msg.as_str());
        match client_msg {
            Ok(client_msg) => self.client_msg(client_msg, ctx),
            Err(error) => {
                println!("Error processing message: {}", error);
                self::Handler::handle(self, ClientResponseMessage::Error { msg }, ctx);
            }
        };
    }

    fn client_msg(&mut self, msg: ClientRequestMessage, ctx: &mut <Self as Actor>::Context) {
        println!("WS: {:?}", msg);
        match msg {
            ClientRequestMessage::Register => self.register(ctx),
            ClientRequestMessage::SetName { name } => self.set_name(name, ctx),
            ClientRequestMessage::SetAvatar { avatar } => self.set_avatar(avatar, ctx),
            ClientRequestMessage::JoinRoom {
                room_name,
                password,
                password_is_hash,
            } => self.join_room(room_name, password, password_is_hash, ctx),
            ClientRequestMessage::LeaveRoom { room_name } => self.leave_room(room_name, ctx),
            ClientRequestMessage::Vote { room_name, size } => self.vote(room_name, size, ctx),
            ClientRequestMessage::NewVote { room_name } => self.new_vote(room_name),
            ClientRequestMessage::Randomize { room_name } => self.randomize(room_name),
            ClientRequestMessage::ChangeScale { room_name, selected_scale_name: selected_scale_name }
                => self.change_scale(room_name, selected_scale_name),
            ClientRequestMessage::UpdateActive { room_name, user_id, active }
                => self.update_active(room_name, user_id, active, ctx)
        }
    }

    fn register(&mut self, ctx: &mut <Self as Actor>::Context) {
        self.notify_data_updated(ctx);
    }

    fn set_name(&mut self, name: String, ctx: &mut <Self as Actor>::Context) {
        self.user.name = name;
        self.notify_data_updated(ctx);
    }

    fn update_active(&mut self,
                     room_name: String,
                     user_id: String,
                     active: bool,
                     ctx: &mut <Self as Actor>::Context) {
        self.room_manager.do_send(RoomMessage::UpdateActive { room_name, user_id, active });
    }

    fn set_avatar(&mut self, avatar: String, ctx: &mut <Self as Actor>::Context) {
        self.user.set_avatar(&avatar);
        self.notify_data_updated(ctx);
    }

    fn notify_data_updated(&mut self, ctx: &mut <Self as Actor>::Context) {
        let user = self.user.clone();
        self::Handler::handle(self, ClientResponseMessage::OwnData { user }, ctx);

        let user = self.user.clone();
        self.room_manager.do_send(RoomMessage::UserUpdated { user });
    }

    fn join_room(
        &mut self,
        room_name: String,
        password: String,
        password_is_hash: bool,
        ctx: &mut <Self as Actor>::Context,
    ) {
        let user = self.user.clone();
        let recipient = ctx.address().recipient();
        let msg = RoomMessage::JoinRoom {
            room_name,
            password,
            password_is_hash,
            user,
            recipient,
        };
        self.room_manager.do_send(msg);
    }

    fn leave_room(&mut self, room_name: String, _ctx: &mut <Self as Actor>::Context) {
        let msg = RoomMessage::LeaveRoom {
            user_id: self.user.user_id.clone(),
            room_name,
        };
        self.room_manager.do_send(msg);
    }

    fn vote(&mut self, room_name: String, size: String, _ctx: &mut <Self as Actor>::Context) {
        let msg = RoomMessage::Vote {
            room_name,
            user_id: self.user.user_id.clone(),
            size,
        };
        self.room_manager.do_send(msg);
    }

    fn new_vote(&self, room_name: String) {
        let msg = RoomMessage::NewVote {
            room_name,
            user_id: self.user.user_id.clone(),
        };
        self.room_manager.do_send(msg);
    }

    fn user_left(&mut self) {
        let msg = RoomMessage::UserLeft {
            user_id: self.user.user_id.clone(),
        };
        self.room_manager.do_send(msg);
    }

    fn randomize(&self, room_name: String) {
        let msg = RoomMessage::Randomize {
            room_name,
        };
        self.room_manager.do_send(msg);
    }
    fn change_scale(&self, room_name: String, selected_scale_name: String) {
        let msg = RoomMessage::ChangeScale {
            room_name,
            selected_scale_name: selected_scale_name,
        };
        self.room_manager.do_send(msg);
    }

    /// helper method that sends ping to client on a fixed interval
    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                act.user_left();
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Handler<ClientResponseMessage> for ClientActor {
    type Result = ();

    fn handle(
        &mut self,
        server_msg: ClientResponseMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let msg = serde_json::to_string(&server_msg);
        match msg {
            Ok(msg) => ctx.text(msg),
            Err(err) => println!(
                "ClientActor: error sending data back to user: {}. Error: {}",
                &self.user.user_id, err
            ),
        }
    }
}
