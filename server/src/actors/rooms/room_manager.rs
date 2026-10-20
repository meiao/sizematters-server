/*
 * SizeMatters - a ticket sizing util
 * Copyright (C) 2025 Andre Onuki
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

mod room_factory;

use crate::actors::messages::RoomMessages;
use actix::prelude::*;
use actix::Actor;
use regex::Regex;
use sizematters_shared::messages::ClientResponseMessage;
use sizematters_shared::UserData;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Room manager. This is an actor that knows about all the created rooms and where each user is.
pub struct RoomManagerActor {
    rooms: HashMap<Arc<String>, Recipient<RoomMessages>>,
    user_room_map: HashMap<Arc<String>, HashSet<Arc<String>>>,
    room_name_validator: Regex,
}

impl Actor for RoomManagerActor {
    type Context = Context<Self>;
}

impl RoomManagerActor {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            user_room_map: HashMap::new(),
            room_name_validator: Regex::new(r"^[-_a-zA-Z0-9]{1,50}$").unwrap(),
        }
    }
}

impl Handler<RoomMessages> for RoomManagerActor {
    type Result = ();

    fn handle(&mut self, msg: RoomMessages, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            RoomMessages::CreateRoom { .. } => {
                self.create_room(msg, ctx);
            }
            RoomMessages::JoinRoom {
                ref room_name,
                password: _,
                ref user,
                password_is_hash: _,
                ref recipient,
            } => {
                self.join_room(
                    room_name.to_owned(),
                    user.user_id.to_owned(),
                    recipient.clone(),
                    msg,
                    ctx,
                );
            }
            RoomMessages::UserUpdated { user } => self.user_updated(user),
            RoomMessages::LeaveRoom { user_id, room_name } => self.leave_room(user_id, room_name),
            RoomMessages::UserLeft { user_id } => self.user_left(user_id),
            RoomMessages::SpecificMessage { ref room_name, .. } => {
                self.forward(room_name.clone(), msg)
            }
            RoomMessages::RoomClosing { room_name } => self.room_closing(room_name),
            RoomMessages::UserJoined {
                user_name,
                room_name,
            } => self.user_joined(user_name, room_name),
            #[allow(unreachable_patterns)]
            _ => {}
        };
    }
}

impl RoomManagerActor {
    fn join_room(
        &mut self,
        room_name: Arc<String>,
        user_id: String,
        recipient: Recipient<ClientResponseMessage>,
        msg: RoomMessages,
        _ctx: &mut Context<Self>,
    ) {
        if !self.room_name_validator.is_match(&room_name) {
            self.notify_user(&user_id, &recipient, ClientResponseMessage::InvalidRoomName);
            return;
        }

        if !self.rooms.contains_key(&room_name) {
            self.notify_user(
                &user_id,
                &recipient,
                ClientResponseMessage::Error {
                    msg: "Room not found.".to_string(),
                },
            );
            return;
        }
        match self.user_room_map.get(&user_id) {
            None => {
                let room = self.rooms.get(&room_name).unwrap();
                room.do_send(msg);
            }
            Some(_) => {
                println!("RoomManager: User trying to join a second room.");
                self.notify_user(
                    &user_id,
                    &recipient,
                    ClientResponseMessage::CannotJoinMultipleRooms,
                );
            }
        }
    }

    fn create_room(&mut self, create_room_msg: RoomMessages, ctx: &mut Context<Self>) {
        let RoomMessages::CreateRoom {
            room_name,
            password,
            password_is_hash,
            room_type,
            user,
            recipient,
        } = create_room_msg
        else {
            return;
        };
        let user_id = &user.user_id;
        if !self.room_name_validator.is_match(&room_name) {
            self.notify_user(user_id, &recipient, ClientResponseMessage::InvalidRoomName);
            return;
        }

        // Creating a room that already exists joins it instead. This lets clients restore
        // their saved rooms without knowing whether those rooms are still alive.
        if self.rooms.contains_key(&room_name) {
            let user_id = user.user_id.clone();
            let join_msg = RoomMessages::JoinRoom {
                room_name: room_name.clone(),
                password,
                password_is_hash,
                user,
                recipient: recipient.clone(),
            };
            self.join_room(room_name, user_id, recipient, join_msg, ctx);
            return;
        }
        let room_manager = ctx.address().recipient();
        let room = room_factory::create(
            room_name.clone(),
            password.clone(),
            password_is_hash,
            room_type,
            room_manager,
        );
        room.do_send(RoomMessages::JoinRoom {
            room_name: room_name.clone(),
            password,
            password_is_hash,
            user,
            recipient,
        });
        self.rooms.insert(room_name, room);
    }

    fn leave_room(&mut self, user_id: Arc<String>, room_name: Arc<String>) {
        match self.user_room_map.get_mut(&user_id) {
            None => (),
            Some(rooms) => {
                let removed = rooms.remove(&room_name);
                if removed {
                    self.notify_room(&room_name, RoomMessages::UserLeft { user_id });
                }
            }
        };
    }

    fn user_left(&mut self, user_id: Arc<String>) {
        let rooms = self.user_room_map.remove(&user_id);
        match rooms {
            None => {}
            Some(rooms) => rooms.iter().for_each(|room| {
                self.notify_room(
                    room,
                    RoomMessages::UserLeft {
                        user_id: user_id.clone(),
                    },
                );
            }),
        }
    }

    fn user_updated(&mut self, user: UserData) {
        let rooms = self.user_room_map.get(&user.user_id);
        match rooms {
            None => {}
            Some(rooms) => {
                rooms.iter().for_each(|room| {
                    self.notify_room(room, RoomMessages::UserUpdated { user: user.clone() })
                });
            }
        }
    }

    fn forward(&mut self, room_name: Arc<String>, msg: RoomMessages) {
        match self.rooms.get(&room_name) {
            None => println!("RoomManager: User tried to send a message to an unknown room."),
            Some(room) => room.do_send(msg),
        }
    }

    fn room_closing(&mut self, room_name: Arc<String>) {
        self.rooms.remove(&room_name);
    }

    fn user_joined(&mut self, user_name: Arc<String>, room_name: Arc<String>) {
        self.user_room_map
            .entry(user_name)
            .or_default()
            .insert(room_name);
    }

    fn notify_room(&self, room_name: &Arc<String>, msg: RoomMessages) {
        let room = self.rooms.get(room_name);
        match room {
            None => println!("RoomManager: Unable to find room to send message to"),
            Some(room) => room.do_send(msg),
        }
    }

    fn notify_user(
        &mut self,
        user_id: &str,
        recipient: &Recipient<ClientResponseMessage>,
        msg: ClientResponseMessage,
    ) {
        if let Err(err) = recipient.try_send(msg) {
            println!("RoomManager: Unable to reach ClientActor.\nError: {}", err);
            self.user_left(Arc::new(user_id.to_owned()));
        }
    }
}
