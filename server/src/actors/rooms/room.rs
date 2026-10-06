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

use crate::actors::messages::RoomMessages;
use actix::{Actor, ActorContext, Context, Handler, Recipient};
use serde::de::DeserializeOwned;
use sizematters_shared::messages::ClientResponseMessage;
use sizematters_shared::UserData;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Logic that is specific to a type of room. `T` is the type of the room specific messages.
///
/// The room takes care of the common work (users, passwords, notifying users) and calls these hooks
/// so the behavior can react to it. The hooks receive a read-only view of the room's state.
pub(super) trait RoomBehavior<T> {
    /// Called after `user_id` was added to the room.
    fn on_join_room(&mut self, user_id: &Arc<String>, room: &mut RoomState);

    /// Called after `user_id` was removed from the room.
    fn on_leave_room(&mut self, user_id: &Arc<String>, room: &mut RoomState);

    /// Called when the last user left and the room is about to stop.
    fn on_room_closing(&mut self, room: &mut RoomState) {}

    fn on_specific_message(&mut self, user_id: Arc<String>, msg: T, room: &mut RoomState);
}

pub(super) struct Room<T: 'static> {
    behavior: Box<dyn RoomBehavior<T>>,
    state: RoomState,
}

impl<T: 'static> Room<T> {
    pub(super) fn new(
        name: Arc<String>,
        password: Arc<String>,
        password_is_hash: bool,
        room_manager: Recipient<RoomMessages>,
        behavior: Box<dyn RoomBehavior<T>>,
    ) -> Self {
        Room {
            behavior,
            state: RoomState {
                name,
                hashed_password: compute_password(password, password_is_hash),
                user_map: HashMap::new(),
                spectators: HashSet::new(),
                room_manager,
            },
        }
    }
}

impl<T: 'static> Actor for Room<T> {
    type Context = Context<Self>;
}

impl<T: DeserializeOwned + 'static> Handler<RoomMessages> for Room<T> {
    type Result = ();

    fn handle(&mut self, msg: RoomMessages, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            RoomMessages::JoinRoom {
                password,
                password_is_hash,
                user,
                recipient,
                ..
            } => self.join_room(password, password_is_hash, user, recipient),
            RoomMessages::LeaveRoom { user_id, .. } => self.leave_room(user_id, ctx),
            RoomMessages::SpecificMessage {
                user_id, payload, ..
            } => self.process_message(user_id, payload),
            RoomMessages::UserUpdated { user } => self.user_updated(user),
            _ => println!("Room: Unhandled message."),
        }
    }
}

impl<T: DeserializeOwned + 'static> Room<T> {
    fn join_room(
        &mut self,
        password: Arc<String>,
        password_is_hash: bool,
        user: UserData,
        recipient: Recipient<ClientResponseMessage>,
    ) {
        let user_id = Arc::new(user.user_id.clone());
        let hashed_password = compute_password(password, password_is_hash);
        let room_name = self.state.room_name();

        if self.state.user_map.contains_key(&user_id) {
            let msg = ClientResponseMessage::AlreadyInRoom { room_name };
            self.state.send_to_user(&user_id, &recipient, msg);
        } else if self.state.hashed_password != hashed_password {
            let msg = ClientResponseMessage::WrongPassword { room_name };
            self.state.send_to_user(&user_id, &recipient, msg);
        } else {

            let connection_info = ConnectionInfo {
                user: UserInfo::from(user.clone()),
                recipient,
            };
            self.state.user_map.insert(user_id.clone(), connection_info);
            self.state.spectators.insert(user_id.clone());
            self.notify_user_joined(user, &user_id, room_name);
            let join_msg = ClientResponseMessage::RoomJoined {
                room_name: self.state.room_name(),
                hashed_password: self.state.hashed_password(),
                users: self.state.users(),
            };
            self.state.notify_user(&user_id, join_msg);

            self.behavior.on_join_room(&user_id, &mut self.state);
        }
    }

    fn leave_room(&mut self, user_id: Arc<String>, ctx: &mut Context<Self>) {
        let msg = ClientResponseMessage::UserLeft {
            user_id: (*user_id).clone(),
            room_name: self.state.room_name(),
        };
        self.state.notify_users(msg);

        self.state.user_map.remove(&user_id);
        self.state.spectators.remove(&user_id);
        self.behavior.on_leave_room(&user_id, &mut self.state);

        if self.state.user_map.is_empty() {
            self.behavior.on_room_closing(&mut self.state);
            let msg = RoomMessages::RoomClosing {
                room_name: self.state.name.clone(),
            };
            self.state.notify_manager(msg);
            ctx.stop();
        }
    }

    fn user_updated(&mut self, user: UserData) {
        match self.state.user_map.get_mut(&user.user_id) {
            None => {
                let room_name = self.state.name.clone();
                let user_id = Arc::new(user.user_id);
                self.state
                    .notify_manager(RoomMessages::LeaveRoom { room_name, user_id });
            }
            Some(conn_info) => {
                conn_info.user = UserInfo::from(user.clone());
                self.state
                    .notify_users(ClientResponseMessage::UserUpdated { user });
            }
        };
    }

    fn process_message(&mut self, user_id: Arc<String>, payload: String) {
        match serde_json::from_str::<T>(payload.as_str()) {
            Ok(msg) => self.behavior.on_specific_message(user_id, msg, &mut self.state),
            Err(_) => self.state.notify_user(
                &user_id,
                ClientResponseMessage::Error {
                    msg: "Unable to read message.".to_string(),
                },
            ),
        };
    }

    fn notify_user_joined(&mut self, user: UserData, user_id: &Arc<String>, room_name: String) {
        let user_joined_msg = ClientResponseMessage::UserJoined {
            room_name,
            user: user.clone(),
        };
        self.state.notify_users(user_joined_msg);
        let user_joined_msg = RoomMessages::UserJoined {
            user_name: user_id.clone(),
            room_name: self.state.name.clone(),
        };
        self.state.room_manager.do_send(user_joined_msg);
    }
}

/// The state that is common to every type of room.
pub(super) struct RoomState {
    name: Arc<String>,
    hashed_password: Arc<String>,
    user_map: HashMap<Arc<String>, ConnectionInfo>,
    spectators: HashSet<Arc<String>>,
    room_manager: Recipient<RoomMessages>,
}

impl RoomState {
    pub(super) fn room_name(&self) -> String {
        (*self.name).clone()
    }

    pub(super) fn hashed_password(&self) -> String {
        (*self.hashed_password).clone()
    }

    pub(super) fn user_ids(&self) -> impl Iterator<Item = &Arc<String>> {
        self.user_map.keys()
    }

    pub(super) fn users(&self) -> Vec<UserData> {
        self.user_map
            .values()
            .map(|conn_info| UserData::from(&conn_info.user))
            .collect()
    }

    pub(super) fn user_count(&self) -> usize {
        self.user_map.len()
    }

    pub(super) fn contains_user(&self, user_id: &Arc<String>) -> bool {
        self.user_map.contains_key(user_id)
    }

    pub(super) fn is_spectator(&self, user_id: &Arc<String>) -> bool {
        self.spectators.contains(user_id)
    }

    pub(super) fn notify_user(&self, user_id: &Arc<String>, msg: ClientResponseMessage) {
        match self.user_map.get(user_id) {
            None => println!("RoomActor: User not found in room."),
            Some(conn_info) => self.send_to_user(user_id, &conn_info.recipient, msg),
        }
    }

    pub(super) fn notify_users(&self, msg: ClientResponseMessage) {
        for (user_id, conn_info) in self.user_map.iter() {
            self.send_to_user(user_id, &conn_info.recipient, msg.clone());
        }
    }

    pub(super) fn remove_spectator(&mut self, user_id: Arc<String>) {
        self.spectators.remove(&user_id);
    }
    
    pub(super) fn add_spectator(&mut self, user_id: Arc<String>) {
        self.spectators.insert(user_id);
    }
    
    pub(super) fn spectator_count(&self) -> usize {
        self.spectators.len()
    }

    pub(super) fn spectators(&self) -> Vec<String> {
        self.spectators.iter().map(|user_id| (**user_id).clone()).collect()
    }

    fn send_to_user(
        &self,
        user_id: &Arc<String>,
        recipient: &Recipient<ClientResponseMessage>,
        msg: ClientResponseMessage,
    ) {
        if let Err(err) = recipient.try_send(msg) {
            println!("RoomActor: Unable to reach ClientActor.\nError: {}", err);
            self.remove_user(user_id.clone());
        }
    }

    fn remove_user(&self, user_id: Arc<String>) {
        self.notify_manager(RoomMessages::UserLeft { user_id });
    }

    fn notify_manager(&self, msg: RoomMessages) {
        if let Err(err) = self.room_manager.try_send(msg) {
            println!("RoomActor: Unable to reach room manager.\nError: {}", err);
        }
    }
}

struct ConnectionInfo {
    user: UserInfo,
    recipient: Recipient<ClientResponseMessage>,
}

/// Same content as `UserData`, but the strings are shared to avoid copying them around.
#[derive(Clone, Debug)]
struct UserInfo {
    user_id: Arc<String>,
    name: Arc<String>,
    gravatar_id: Arc<String>,
}

impl From<UserData> for UserInfo {
    fn from(user: UserData) -> Self {
        UserInfo {
            user_id: Arc::new(user.user_id),
            name: Arc::new(user.name),
            gravatar_id: Arc::new(user.gravatar_id),
        }
    }
}

impl From<&UserInfo> for UserData {
    fn from(user: &UserInfo) -> Self {
        UserData {
            user_id: (*user.user_id).clone(),
            name: (*user.name).clone(),
            gravatar_id: (*user.gravatar_id).clone(),
        }
    }
}

fn compute_password(password: Arc<String>, password_is_hash: bool) -> Arc<String> {
    if password_is_hash {
        password
    } else {
        Arc::new(format!("{:x}", md5::compute(password.as_bytes())))
    }
}
