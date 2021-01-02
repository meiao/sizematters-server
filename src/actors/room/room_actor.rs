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

mod join_room;
mod leave_room;
mod vote;

use crate::actors::messages::{ClientResponseMessage, RoomMessage};
use crate::data::UserData;
use actix::{Actor, Context, Handler, Recipient};
use std::collections::HashMap;

pub struct RoomActor {
    name: String,
    hashed_password: String,
    user_map: HashMap<String, ConnectionInfo>,
    vote_map: HashMap<String, u64>,
    room_manager: Recipient<RoomMessage>,
    voting_over: bool,
}

impl RoomActor {
    pub fn new(
        name: String,
        password: String,
        password_is_hash: bool,
        room_manager: Recipient<RoomMessage>,
    ) -> RoomActor {
        let hashed_password = compute_password(password, password_is_hash);
        RoomActor {
            name,
            hashed_password,
            user_map: HashMap::new(),
            vote_map: HashMap::new(),
            room_manager,
            voting_over: false,
        }
    }
}

impl Actor for RoomActor {
    type Context = Context<Self>;
}

impl Handler<RoomMessage> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: RoomMessage, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            RoomMessage::JoinRoom {
                password,
                password_is_hash,
                user,
                recipient,
                ..
            } => self.join_room(password, password_is_hash, user, recipient),
            RoomMessage::LeaveRoom { user_id, .. } => self.leave_room(user_id, ctx),
            RoomMessage::Vote { user_id, size, .. } => self.vote(user_id, size),
            RoomMessage::NewVote { user_id, .. } => self.new_vote(user_id),
            RoomMessage::UserUpdated { user } => self.user_updated(user),
            _ => println!("RoomActor: Unhandled message."),
        }
    }
}

impl RoomActor {
    fn user_updated(&mut self, user: UserData) {
        match self.user_map.get_mut(&user.user_id) {
            None => println!("RoomActor: Updating user not found in room."),
            Some(conn_info) => {
                conn_info.user = user.clone();
                self.notify_users(ClientResponseMessage::UserUpdated { user });
            }
        };
    }

    fn notify_users(&self, msg: ClientResponseMessage) {
        for (user_id, conn_info) in self.user_map.iter() {
            self.notify_user(user_id, &conn_info.recipient, msg.clone());
        }
    }

    fn notify_user(
        &self,
        user_id: &str,
        recipient: &Recipient<ClientResponseMessage>,
        msg: ClientResponseMessage,
    ) {
        if let Err(err) = recipient.do_send(msg) {
            println!("RoomActor: Unable to reach ClientActor.\nError: {}", err);
            self.remove_user(user_id.to_owned());
        }
    }

    fn notify_manager(&self, msg: RoomMessage) {
        if let Err(err) = self.room_manager.do_send(msg) {
            println!("RoomActor: Unable to reach room manager.\nError: {}", err);
        }
    }

    fn remove_user(&self, user_id: String) {
        let msg = RoomMessage::UserLeft { user_id };
        self.notify_manager(msg);
    }
}

fn compute_password(password: String, password_is_hash: bool) -> String {
    if password_is_hash {
        password
    } else {
        format!("{:x}", md5::compute(password))
    }
}

struct ConnectionInfo {
    user: UserData,
    recipient: Recipient<ClientResponseMessage>,
}
