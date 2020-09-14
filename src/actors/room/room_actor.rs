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
    ClientResponseMessage, JoinRoom, LeaveRoom, NewVote, UserUpdated, Vote,
};
use crate::actors::RoomManagerActor;
use crate::data::UserData;
use actix::{Actor, ActorContext, Addr, Context, Handler, Recipient};
use std::borrow::Borrow;
use std::collections::HashMap;

pub struct RoomActor {
    name: String,
    hashed_password: String,
    user_map: HashMap<String, ConnectionInfo>,
    vote_map: HashMap<String, u64>,
    room_manager: Addr<RoomManagerActor>,
    voting_over: bool,
}

impl RoomActor {
    pub fn new(
        name: String,
        password: String,
        password_is_hash: bool,
        room_manager: Addr<RoomManagerActor>,
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

impl Handler<JoinRoom> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: JoinRoom, ctx: &mut Context<Self>) -> Self::Result {
        let user = msg.user;
        let recipient = msg.recipient;
        let user_id = user.user_id.clone();
        let hashed_password = compute_password(msg.password, msg.password_is_hash);

        if self.user_map.contains_key(&user_id) {
            recipient.do_send(ClientResponseMessage::AlreadyInRoom {
                room_name: msg.room_name,
            });
        } else if !(self.hashed_password.eq(&hashed_password)) {
            recipient.do_send(ClientResponseMessage::WrongPassword {
                room_name: msg.room_name,
            });
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
            joiner.do_send(join_msg);
        };
    }
}

impl Handler<LeaveRoom> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoom, ctx: &mut Context<Self>) -> Self::Result {
        let user_id = msg.user_id;
        let user_left_msg = ClientResponseMessage::UserLeft {
            user_id: user_id.clone(),
            room_name: self.name.clone(),
        };
        self.notify_users(user_left_msg);

        self.user_map.remove(&user_id);
        self.vote_map.remove(&user_id);

        let msg = ClientResponseMessage::VotesCast {
            votes_cast: self.vote_map.len(),
            room_name: self.name.clone(),
        };
        self.notify_users(msg);

        if self.user_map.is_empty() {
            let msg = RoomMessage::RoomClosing {
                room_name: self.name.clone(),
            };
            self.room_manager.borrow().do_send(msg);
            ctx.stop();
        }
    }
}

impl Handler<Vote> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: Vote, ctx: &mut Context<Self>) -> Self::Result {
        let user_id = msg.user_id;
        let size = msg.size;
        if self.voting_over() {
            match self.user_map.get(&user_id) {
                None => println!("User tried to cast vote in a room he is not in."),
                Some(user) => {
                    let msg = ClientResponseMessage::VotingOver;
                    user.recipient.borrow().do_send(msg);
                }
            }
        } else {
            match self.user_map.get(&user_id) {
                None => println!("User tried to cast vote in a room he is not in."),
                Some(user) => {
                    let room_name = self.name.clone();
                    let msg = ClientResponseMessage::OwnVote { room_name, size };
                    user.recipient.borrow().do_send(msg);
                }
            }

            let already_voted = self.vote_map.contains_key(&user_id);
            self.vote_map.insert(user_id, size);

            if !already_voted {
                let msg = ClientResponseMessage::VotesCast {
                    room_name: self.name.clone(),
                    votes_cast: self.vote_map.len(),
                };
                self.notify_users(msg);

                if self.voting_over() {
                    let room_name = self.name.clone();
                    let votes = self.vote_map.clone();
                    let msg = ClientResponseMessage::VoteResults { room_name, votes };
                    self.notify_users(msg);
                }
            }
        }
    }
}

impl Handler<NewVote> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: NewVote, ctx: &mut Context<Self>) -> Self::Result {
        if !self.user_map.contains_key(&msg.user_id) {
            println!("User tried to request new vote in a room they is not in.");
            return;
        }

        self.voting_over = false;
        self.vote_map.clear();

        self.notify_users(ClientResponseMessage::NewVote {
            room_name: self.name.clone(),
        });
    }
}

impl Handler<UserUpdated> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: UserUpdated, ctx: &mut Context<Self>) -> Self::Result {
        let user = msg.user;
        match self.user_map.get_mut(&user.user_id) {
            None => println!("Updating user not found in room."),
            Some(conn_info) => {
                conn_info.user = user.clone();
                self.notify_users(ClientResponseMessage::UserUpdated { user });
            }
        };
    }
}

impl RoomActor {
    fn voting_over(&self) -> bool {
        self.vote_map.len() == self.user_map.len()
    }

    fn notify_users(&self, msg: ClientResponseMessage) {
        self.user_map
            .values()
            .into_iter()
            .map(|conn_info| conn_info.recipient.borrow())
            .map(|recipient| recipient.do_send(msg.clone()))
            .map(|result| result.err()) // in case there are errors sending the message
            .flatten()
            .for_each(|error| println!("Error sending message: {}", error));
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
