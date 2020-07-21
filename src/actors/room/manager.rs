use crate::actors::messages::{ClientResponseMessage, RoomMessage};
use crate::data::UserData;
use actix::prelude::*;
use actix::Actor;
use std::borrow::Borrow;

/// Room manager. This is an actor that knows about all the created rooms and
pub struct RoomManagerActor {}

impl Actor for RoomManagerActor {
    type Context = Context<Self>;
}

impl RoomManagerActor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Handler<RoomMessage> for RoomManagerActor {
    type Result = ();

    fn handle(&mut self, msg: RoomMessage, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            RoomMessage::JoinRoom {
                room_name,
                password,
                user,
                recipient,
            } => {
                self.join_room(room_name, password, user, recipient);
            }
            _ => {}
        };
    }
}

impl RoomManagerActor {
    fn join_room(
        &self,
        room_name: String,
        password: String,
        user: UserData,
        recipient: Recipient<ClientResponseMessage>,
    ) {
        let msg = ClientResponseMessage::UserEntered { room_name, user };
        recipient.do_send(msg);
    }
}
