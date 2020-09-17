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

use crate::actors::messages::{ClientResponseMessage, RoomMessage};
use crate::actors::room::RoomActor;
use crate::data::UserData;
use actix::prelude::*;
use actix::Actor;
use regex::Regex;
use std::borrow::Borrow;
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};

/// Room manager. This is an actor that knows about all the created rooms and where each user is.
pub struct RoomManagerActor {
    rooms: HashMap<String, Addr<RoomActor>>,
    user_room_map: HashMap<String, HashSet<String>>,
    roomNameValidator: Regex,
}

impl Actor for RoomManagerActor {
    type Context = Context<Self>;
}

impl RoomManagerActor {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            user_room_map: HashMap::new(),
            roomNameValidator: Regex::new(r"^[-_a-zA-Z]+$").unwrap(),
        }
    }
}

impl Handler<RoomMessage> for RoomManagerActor {
    type Result = ();

    fn handle(&mut self, msg: RoomMessage, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            RoomMessage::JoinRoom {
                ref room_name,
                ref password,
                ref user,
                ref password_is_hash,
                ref recipient,
            } => {
                self.join_room(
                    room_name.to_owned(),
                    password.to_owned(),
                    password_is_hash.clone(),
                    user.user_id.to_owned(),
                    recipient.clone(),
                    msg,
                    ctx,
                );
            }
            RoomMessage::UserUpdated { user } => self.user_updated(user),
            RoomMessage::LeaveRoom { user_id, room_name } => self.leave_room(user_id, room_name),
            RoomMessage::UserLeft { user_id } => self.user_left(user_id),
            RoomMessage::Vote { ref room_name, .. } => self.forward(room_name.clone(), msg),
            RoomMessage::NewVote { ref room_name, .. } => self.forward(room_name.clone(), msg),
            RoomMessage::RoomClosing { room_name } => self.room_closing(room_name),
            _ => {}
        };
    }
}

impl RoomManagerActor {
    fn join_room(
        &mut self,
        room_name: String,
        password: String,
        password_is_hash: bool,
        user_id: String,
        recipient: Recipient<ClientResponseMessage>,
        msg: RoomMessage,
        ctx: &mut Context<Self>,
    ) {
        if self.roomNameValidator.is_match(&room_name) {
            if !self.rooms.contains_key(&room_name) {
                self.create_room(room_name.clone(), password, password_is_hash, ctx);
            }
            self.do_join_room(room_name, user_id, msg);
        } else {
            recipient
                .borrow()
                .do_send(ClientResponseMessage::InvalidRoomName);
        }
    }

    fn create_room(
        &mut self,
        room_name: String,
        password: String,
        password_is_hash: bool,
        ctx: &mut Context<Self>,
    ) {
        let room_manager = ctx.address().recipient();
        let room_actor =
            RoomActor::new(room_name.clone(), password, password_is_hash, room_manager).start();
        self.rooms.insert(room_name, room_actor);
    }

    fn do_join_room(&mut self, room_name: String, user_id: String, msg: RoomMessage) {
        match self.user_room_map.get_mut(&user_id) {
            None => println!("RoomManager: User trying to join not found in room manager."),
            Some(room_names) => {
                room_names.insert(room_name.to_owned());
                let room = self.rooms.get(&room_name).unwrap();
                room.do_send(msg);
            }
        }
    }

    fn leave_room(&mut self, user_id: String, room_name: String) {
        match self.user_room_map.get_mut(&user_id) {
            None => println!(
                "RoomManager: {} tried to exit {} which they is not into.",
                &user_id, &room_name
            ),
            Some(rooms) => {
                let _ = rooms.remove(&room_name);
            }
        };

        match self.rooms.get(&room_name) {
            None => println!(
                "RoomManager: {} tried to exit {} which does not exist",
                &user_id, &room_name
            ),
            Some(room) => room.do_send(RoomMessage::LeaveRoom { user_id, room_name }),
        }
    }

    fn user_left(&mut self, user_id: String) {
        let rooms = self.user_room_map.remove(&user_id);
        match rooms {
            None => println!("RoomManager: User left, but no record of his rooms exists."),
            Some(rooms) => {
                rooms
                    .into_iter()
                    .for_each(|room| self.leave_room(user_id.clone(), room));
            }
        }
    }

    fn user_updated(&mut self, user: UserData) {
        if !self.user_room_map.contains_key(&user.user_id) {
            self.user_room_map
                .insert(user.user_id.clone(), HashSet::new());
        }
        let room_names = self.user_room_map.get(&user.user_id).unwrap();
        if !room_names.is_empty() {
            self.notify_rooms(room_names, RoomMessage::UserUpdated { user });
        }
    }

    fn forward(&mut self, room_name: String, msg: RoomMessage) {
        match self.rooms.get(&room_name) {
            None => println!("RoomManager: User tried to send a message to an unknown room."),
            Some(room) => room.do_send(msg),
        }
    }

    fn room_closing(&mut self, room_name: String) {
        self.rooms.remove(&room_name);
    }

    fn notify_rooms(&self, room_names: &HashSet<String>, msg: RoomMessage) {
        room_names
            .iter()
            .map(|room_name| self.rooms.get(room_name))
            .flatten()
            .for_each(|room| room.do_send(msg.clone()));
    }
}
