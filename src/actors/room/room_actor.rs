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

mod join_room;
mod leave_room;
mod vote;

use crate::actors::messages::{ClientResponseMessage, RoomMessage};
use crate::data::UserData;
use crate::data::Scale;
use actix::{Actor, Context, Handler, Recipient};
use std::collections::HashMap;
use rand::Rng;

pub struct RoomActor {
    name: String,
    hashed_password: String,
    active_user_map: HashMap<String, ConnectionInfo>,
    passive_user_map: HashMap<String, ConnectionInfo>,
    vote_map: HashMap<String, String>,
    room_manager: Recipient<RoomMessage>,
    voting_over: bool,
    scale_values: HashMap<String, Scale>,
    selected_scale_name: String,
}

impl RoomActor {
    pub fn new(
        name: String,
        password: String,
        password_is_hash: bool,
        room_manager: Recipient<RoomMessage>,
    ) -> RoomActor {
        let hashed_password = compute_password(password, password_is_hash);

        let mut scale_values = HashMap::new();

        //Add here to expand the scales we support.
        //This should be the ONLY place scales need to be specified!
       scale_values.insert(String::from("fibonacci"),
                           Scale
           {
               name: String::from("fibonacci"),
               displayName: String::from("Fibonacci"),
               values: vec!
               [
                   String::from("0"),
                   String::from("1"),
                   String::from("2"),
                   String::from("3"),
                   String::from("5"),
                   String::from("8"),
                   String::from("13"),
                   String::from("NV")
                   ]
           }
        );

       scale_values.insert(String::from("fistOfFive"),
                           Scale
           {
               name: String::from("fistOfFive"),
               displayName: String::from("FistOfFive"),
               values: vec!
               [
                   String::from("1"),
                   String::from("2"),
                   String::from("3"),
                   String::from("4"),
                   String::from("5"),
                   String::from("NV")
               ]
           }
        );

        RoomActor {
            name,
            hashed_password,
            active_user_map: HashMap::new(),
            passive_user_map: HashMap::new(),
            vote_map: HashMap::new(),
            room_manager,
            voting_over: false,
            scale_values,
            selected_scale_name: String::from("fibonacci")
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
            RoomMessage::Randomize { .. } => self.randomize(),
            RoomMessage::ChangeScale { selected_scale_name: selected_scale, .. } => self.change_scale(selected_scale),
            RoomMessage::UpdateActive { user_id, active, ..  } => self
                .update_active(user_id, active),
            _ => println!("RoomActor: Unhandled message."),
        }
    }
}

impl RoomActor {
    fn user_updated(&mut self, user: UserData) {

        let updating_user = user.clone();
        match self.active_user_map.get_mut(&user.user_id) {
            None => println!("RoomActor: Updating user not active in room."),
            Some(conn_info) => {
                conn_info.user = user.clone();
                self.notify_users(ClientResponseMessage::UserUpdated { user });
            }
        };
        match self.passive_user_map.get_mut(&updating_user.user_id) {
            None => println!("RoomActor: Updating user not passive in room."),
            Some(conn_info) => {
                conn_info.user = updating_user.clone();
                self.notify_users(ClientResponseMessage::UserUpdated { user: updating_user });
            }
        };
    }

    fn notify_users(&self, msg: ClientResponseMessage) {
        for (user_id, conn_info) in self.active_user_map.iter() {
            self.notify_user(user_id, &conn_info.recipient, msg.clone());
        }
        for (user_id, conn_info) in self.passive_user_map.iter() {
            self.notify_user(user_id, &conn_info.recipient, msg.clone());
        }
    }

    fn notify_user(
        &self,
        user_id: &str,
        recipient: &Recipient<ClientResponseMessage>,
        msg: ClientResponseMessage,
    ) {
        if let Err(err) = recipient.try_send(msg) {
            println!("RoomActor: Unable to reach ClientActor.\nError: {}", err);
            self.remove_user(user_id.to_owned());
        }
    }

    fn notify_manager(&self, msg: RoomMessage) {
        if let Err(err) = self.room_manager.try_send(msg) {
            println!("RoomActor: Unable to reach room manager.\nError: {}", err);
        }
    }

    fn remove_user(&self, user_id: String) {
        let msg = RoomMessage::UserLeft { user_id };
        self.notify_manager(msg);
    }

    fn randomize(&self) {
        let users : Vec<String> = self.active_user_map.keys().cloned().collect();
        let mut user_index = 0;
        if self.active_user_map.len() > 1 {
            user_index = rand::rng().random_range(0..self.active_user_map.len());
        }
        let selected_user = users.get(user_index);
        let room_name = self.name.clone();
        match selected_user {
            None => println!("RoomActor: User not found in room."),
            Some(user_id) => {
                let selected_user_id = user_id.clone();
                self.notify_users(ClientResponseMessage::Randomized { room_name, selected_user_id });
            }
        }
    }
    fn change_scale(&mut self, selected_scale_name: String)
    {
        let selected_scale = self.scale_values.get(&selected_scale_name);

        let room_name = self.name.clone();
        match selected_scale {
            None => println!("RoomActor: selected scale not found for key {}", selected_scale_name),
            Some(selected_scale) => {
                self.selected_scale_name = selected_scale_name;
                self.notify_users(ClientResponseMessage::ScaleChanged { room_name,
                    selected_scale: selected_scale.clone()
                });
            }
        }
    }
    /// The heart of active vs passive.  Shuffles users between the active_user_map and the
    /// passive_user_map per the passed in status.
    fn update_active(&mut self, user_id: String, active: bool)
    {
        let room_name = self.name.clone();
        if active
        {
            if self.passive_user_map.contains_key(&user_id)
            {
                self.active_user_map.insert(String::from(&user_id), self.passive_user_map
                    .remove(&user_id).unwrap());
            }
        }
        else
        {
            if self.active_user_map.contains_key(&user_id)
            {
                self.passive_user_map.insert(String::from(&user_id), self.active_user_map
                    .remove(&user_id).unwrap());

                // If user has voted, remove vote.
                self.vote_map.remove(&user_id);
                self.send_vote_info();
            }
        }
        self.notify_users(ClientResponseMessage::ActiveUpdated { room_name,
            user_id: user_id.clone(), active: active
        });

    }
    fn in_room(
        &self,
        user_id: &str
    ) -> bool
    {
        self.active_user_map.contains_key(user_id) ||
            self.passive_user_map.contains_key(user_id)
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
