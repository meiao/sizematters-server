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
use crate::actors::RoomManagerActor;
use crate::data::UserData;
use actix::prelude::SendError;
use actix::{Actor, ActorContext, Context, Handler, Recipient};
use std::borrow::Borrow;
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
    fn join_room(
        &mut self,
        password: String,
        password_is_hash: bool,
        user: UserData,
        recipient: Recipient<ClientResponseMessage>,
    ) {
        let user_id = user.user_id.clone();
        let hashed_password = compute_password(password, password_is_hash);

        if self.user_map.contains_key(&user_id) {
            let room_name = self.name.clone();
            let msg = ClientResponseMessage::AlreadyInRoom { room_name };
            self.notify_user(&user_id, &recipient, msg);
        } else if !(self.hashed_password.eq(&hashed_password)) {
            let room_name = self.name.clone();
            let msg = ClientResponseMessage::WrongPassword { room_name };
            self.notify_user(&user_id, &recipient, msg);
        } else {
            let user_entered_msg = ClientResponseMessage::UserJoined {
                room_name: self.name.clone(),
                user: user.clone(),
            };
            self.notify_users(user_entered_msg);

            let connection_info = ConnectionInfo { user, recipient };
            self.user_map.insert(user_id.clone(), connection_info);

            let joiner = self.user_map.get(&user_id).unwrap().recipient.borrow();
            let users: Vec<UserData> = self
                .user_map
                .values()
                .map(|conn_info| conn_info.user.clone())
                .collect();
            let join_msg = ClientResponseMessage::RoomJoined {
                room_name: self.name.clone(),
                hashed_password: self.hashed_password.clone(),
                users,
                votes_cast: self.vote_map.len(),
            };
            self.notify_user(&user_id, joiner, join_msg);
        };
    }

    fn leave_room(&mut self, user_id: String, ctx: &mut Context<Self>) {
        let msg = ClientResponseMessage::UserLeft {
            user_id: user_id.clone(),
            room_name: self.name.clone(),
        };
        self.notify_users(msg);

        self.user_map.remove(&user_id);
        self.vote_map.remove(&user_id);

        self.send_vote_info();

        if self.user_map.is_empty() {
            let msg = RoomMessage::RoomClosing {
                room_name: self.name.clone(),
            };
            self.notify_manager(msg);
            ctx.stop();
        }
    }

    fn vote(&mut self, user_id: String, size: u64) {
        if self.voting_over() {
            match self.user_map.get(&user_id) {
                None => println!("RoomActor: User tried to cast vote in a room he is not in."),
                Some(user) => {
                    let msg = ClientResponseMessage::VotingOver;
                    self.notify_user(&user.user.user_id, &user.recipient, msg);
                }
            }
        } else {
            match self.user_map.get(&user_id) {
                None => println!("RoomActor: User tried to cast vote in a room he is not in."),
                Some(user) => {
                    let room_name = self.name.clone();
                    let msg = ClientResponseMessage::OwnVote { room_name, size };
                    self.notify_user(&user.user.user_id, &user.recipient, msg);
                }
            }

            let already_voted = self.vote_map.contains_key(&user_id);
            self.vote_map.insert(user_id, size);

            if !already_voted {
                self.send_vote_info();
            }
        }
    }

    fn send_vote_info(&self) {
        let room_name = self.name.clone();
        if self.voting_over() {
            let votes = self.vote_map.clone();
            let msg = ClientResponseMessage::VoteResults { room_name, votes };
            self.notify_users(msg);
        } else {
            let mut votes = HashMap::new();
            for user_id in self.user_map.keys() {
                let has_voted = self.vote_map.contains_key(user_id);
                votes.insert(user_id.to_owned(), has_voted);
            }
            let msg = ClientResponseMessage::VoteStatus { room_name, votes };
            self.notify_users(msg);
        }
    }

    fn new_vote(&mut self, user_id: String) {
        if !self.user_map.contains_key(&user_id) {
            println!("RoomActor: User tried to request new vote in a room they is not in.");
            return;
        }

        self.voting_over = false;
        self.vote_map.clear();

        self.notify_users(ClientResponseMessage::NewVote {
            room_name: self.name.clone(),
        });
    }

    fn voting_over(&self) -> bool {
        self.vote_map.len() == self.user_map.len()
    }

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
