use actix::prelude::*;
use actix_web_actors::ws;
use serde_json::Error;
use std::collections::HashSet;
use std::time::Duration;
use std::time::Instant;
use uuid::Uuid;

use super::RoomManagerActor;
use crate::actors::messages::{ClientRequestMessage, ClientResponseMessage, UserInfoUpdate};
use crate::actors::user::UserManagerActor;

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
    rooms: HashSet<String>,
    user_id: String,
    room_manager: Addr<RoomManagerActor>,
    user_manager: Addr<UserManagerActor>,
}

impl Actor for ClientActor {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start. We start the heartbeat process here.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
        let user_id = self.user_id.clone();
        let recipient = ctx.address().recipient();
        self.user_manager
            .do_send(UserInfoUpdate::Register { user_id, recipient })
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
            Ok(ws::Message::Binary(bin)) => {} // ignore binary
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl ClientActor {
    pub fn new(room_manager: Addr<RoomManagerActor>, user_manager: Addr<UserManagerActor>) -> Self {
        Self {
            last_heartbeat: Instant::now(),
            rooms: HashSet::new(),
            user_id: Uuid::new_v4().simple().to_string(),
            room_manager,
            user_manager,
        }
    }

    fn text(&mut self, msg: String, ctx: &mut <Self as Actor>::Context) {
        println!("WS: {:?}", msg);
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
        match msg {
            ClientRequestMessage::Register { .. } => self.register(ctx),
            ClientRequestMessage::SetName { name } => self.set_name(name, ctx),
            ClientRequestMessage::SetAvatar { avatar } => self.set_avatar(avatar, ctx),
            ClientRequestMessage::JoinRoom { .. } => self.join_room(msg, ctx),
            ClientRequestMessage::LeaveRoom { .. } => self.leave_room(msg, ctx),
            ClientRequestMessage::Vote { .. } => self.vote(msg, ctx),
        }
    }

    fn register(&self, ctx: &mut <Self as Actor>::Context) {
        let msg = UserInfoUpdate::Register {
            user_id: self.user_id.clone(),
            recipient: ctx.address().recipient(),
        };
        self.user_manager.do_send(msg);
    }

    fn set_name(&mut self, name: String, ctx: &mut <Self as Actor>::Context) {
        let msg = UserInfoUpdate::SetName {
            user_id: self.user_id.clone(),
            name,
            recipient: ctx.address().recipient(),
        };
        self.user_manager.do_send(msg);
    }

    fn set_avatar(&mut self, gravatar_id: String, ctx: &mut <Self as Actor>::Context) {
        let msg = UserInfoUpdate::SetAvatar {
            user_id: self.user_id.clone(),
            gravatar_id,
            recipient: ctx.address().recipient(),
        };
        self.user_manager.do_send(msg);
    }

    fn join_room(&mut self, msg: ClientRequestMessage, ctx: &mut <Self as Actor>::Context) {
        if let ClientRequestMessage::JoinRoom {
            room_name,
            password,
        } = msg
        {}
    }

    fn leave_room(&mut self, msg: ClientRequestMessage, ctx: &mut <Self as Actor>::Context) {
        if let ClientRequestMessage::LeaveRoom { room_name } = msg {}
    }

    fn vote(&mut self, msg: ClientRequestMessage, ctx: &mut <Self as Actor>::Context) {
        if let ClientRequestMessage::Vote { room_name, size } = msg {}
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

///
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
            Err(_) => println!("Error sending data back to user: {}", &self.user_id),
        }
    }
}
