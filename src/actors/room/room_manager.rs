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

use crate::actors::messages::{
    ClientResponseMessage, JoinRoom, LeaveRoom, NewVote, RoomClosing, UserLeft, UserUpdated, Vote,
};
use crate::actors::room::RoomActor;
use actix::prelude::*;
use actix::Actor;
use regex::Regex;
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

impl Handler<JoinRoom> for RoomManagerActor {
    type Result = ();

    fn handle(&mut self, msg: JoinRoom, ctx: &mut Context<Self>) -> Self::Result {
        let room_name = msg.room_name.clone();
        let user_id = msg.user.user_id.clone();
        if self.roomNameValidator.is_match(&room_name) {
            if !self.rooms.contains_key(&room_name) {
                let password = msg.password.clone();
                let password_is_hash = msg.password_is_hash;
                self.create_room(room_name.clone(), password, password_is_hash, ctx);
            }
            self.join_room(room_name, user_id, msg);
        } else {
            msg.recipient
                // TODO investigate whether this borrow is needed
                // .borrow()
                .do_send(ClientResponseMessage::InvalidRoomName);
        }
    }
}

impl RoomManagerActor {
    fn create_room(
        &mut self,
        room_name: String,
        password: String,
        password_is_hash: bool,
        ctx: &mut Context<Self>,
    ) {
        let room_manager = ctx.address();
        let room_actor =
            RoomActor::new(room_name.clone(), password, password_is_hash, room_manager).start();
        self.rooms.insert(room_name, room_actor);
    }

    fn join_room(&mut self, room_name: String, user_id: String, msg: JoinRoom) {
        match self.user_room_map.get_mut(&user_id) {
            None => println!("User trying to join not found in room manager."),
            Some(room_names) => {
                room_names.insert(room_name.to_owned());
                let room = self.rooms.get(&room_name).unwrap();
                room.do_send(msg);
            }
        }
    }
}

impl Handler<LeaveRoom> for RoomManagerActor {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoom, ctx: &mut Context<Self>) -> Self::Result {
        let user_id = msg.user_id;
        let room_name = msg.room_name;
        self.leave_room(user_id, room_name);
    }
}

impl RoomManagerActor {
    fn leave_room(&mut self, user_id: String, room_name: String) {
        match self.user_room_map.get_mut(&user_id) {
            None => println!(
                "{} tried to exit {} which they is not into.",
                &user_id, &room_name
            ),
            Some(rooms) => {
                let _ = rooms.remove(&room_name);
            }
        };

        match self.rooms.get(&room_name) {
            None => println!(
                "{} tried to exit {} which does not exist",
                &user_id, &room_name
            ),
            Some(room) => room.do_send(RoomMessage::LeaveRoom { user_id, room_name }),
        }
    }
}

impl Handler<UserLeft> for RoomManagerActor {
    type Result = ();

    fn handle(&mut self, msg: UserLeft, ctx: &mut Context<Self>) -> Self::Result {
        let user_id = msg.user_id;
        let rooms = self.user_room_map.remove(&user_id);
        match rooms {
            None => println!("User left, but no record of his rooms exists."),
            Some(rooms) => {
                rooms
                    .into_iter()
                    .for_each(|room| self.leave_room(user_id.clone(), room));
            }
        }
    }
}

impl Handler<UserUpdated> for RoomManagerActor {
    type Result = ();

    fn handle(&mut self, msg: UserUpdated, ctx: &mut Context<Self>) -> Self::Result {
        let user = msg.user;
        if !self.user_room_map.contains_key(&user.user_id) {
            self.user_room_map
                .insert(user.user_id.clone(), HashSet::new());
        }
        let room_names = self.user_room_map.get(&user.user_id).unwrap();
        if !room_names.is_empty() {
            self.notify_rooms(room_names, UserUpdated { user });
        }
    }
}

impl Handler<Vote> for RoomManagerActor {
    type Result = ();

    fn handle(&mut self, msg: Vote, ctx: &mut Context<Self>) -> Self::Result {
        let room_name = msg.room_name.clone();
        match self.rooms.get(&room_name) {
            None => println!("User tried to send a message to an unknown room."),
            Some(room) => room.do_send(msg),
        }
    }
}

impl Handler<NewVote> for RoomManagerActor {
    type Result = ();

    fn handle(&mut self, msg: NewVote, ctx: &mut Context<Self>) -> Self::Result {
        let room_name = msg.room_name.clone();
        match self.rooms.get(&room_name) {
            None => println!("User tried to send a message to an unknown room."),
            Some(room) => room.do_send(msg),
        }
    }
}

impl RoomManagerActor {
    fn notify_rooms(&self, room_names: &HashSet<String>, msg: RoomMessage) {
        room_names
            .iter()
            .map(|room_name| self.rooms.get(room_name))
            .flatten()
            .for_each(|room| room.do_send(msg.clone()));
    }
}

impl Handler<RoomClosing> for RoomManagerActor {
    type Result = ();

    fn handle(&mut self, msg: RoomClosing, ctx: &mut Context<Self>) -> Self::Result {
        let room_name = msg.room_name;
        self.rooms.remove(&room_name);
    }
}
