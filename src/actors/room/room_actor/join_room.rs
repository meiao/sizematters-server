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

use crate::actors::messages::ClientResponseMessage;
use crate::actors::room::room_actor::{compute_password, ConnectionInfo};
use crate::actors::room::RoomActor;
use crate::data::UserData;
use actix::Recipient;
use std::borrow::Borrow;

impl RoomActor {
    pub(super) fn join_room(
        &mut self,
        password: String,
        password_is_hash: bool,
        user: UserData,
        recipient: Recipient<ClientResponseMessage>,
    ) {
        let user_id = user.user_id.clone();
        let hashed_password = compute_password(password, password_is_hash);

        if self.in_room(&user_id)  {
            self.user_already_in_room(&recipient, &user_id);
        } else if !(self.hashed_password.eq(&hashed_password)) {
            self.wrong_password(&recipient, &user_id);
        } else {
            self.do_join_room(user, recipient, &user_id);
        };
    }

    fn user_already_in_room(
        &mut self,
        recipient: &Recipient<ClientResponseMessage>,
        user_id: &String,
    ) {
        let room_name = self.name.clone();
        let msg = ClientResponseMessage::AlreadyInRoom { room_name };
        self.notify_user(&user_id, &recipient, msg);
    }

    fn wrong_password(&mut self, recipient: &Recipient<ClientResponseMessage>, user_id: &String) {
        let room_name = self.name.clone();
        let msg = ClientResponseMessage::WrongPassword { room_name };
        self.notify_user(&user_id, &recipient, msg);
    }

    fn do_join_room(
        &mut self,
        user: UserData,
        recipient: Recipient<ClientResponseMessage>,
        user_id: &String,
    ) {
        let user_entered_msg = ClientResponseMessage::UserJoined {
            room_name: self.name.clone(),
            user: user.clone(),
        };
        self.notify_users(user_entered_msg);

        let connection_info = ConnectionInfo { user, recipient };
        self.active_user_map.insert(user_id.clone(), connection_info);

        let joiner = self.active_user_map.get(user_id).unwrap().recipient.borrow();
        let mut users: Vec<UserData> = self
            .active_user_map
            .values()
            .map(|conn_info| conn_info.user.clone())
            .collect();

        let mut passive_users: Vec<UserData> = self
            .passive_user_map
            .values()
            .map(|conn_info| conn_info.user.clone())
            .collect();
        users.append(&mut passive_users);

        let join_msg = ClientResponseMessage::RoomJoined {
            room_name: self.name.clone(),
            hashed_password: self.hashed_password.clone(),
            users,
            votes_cast: self.vote_map.len(),
            scale_values: self.scale_values.clone(),
            selected_scale_name: self.selected_scale_name.clone()
        };
        self.notify_user(&user_id, joiner, join_msg);
    }
}
