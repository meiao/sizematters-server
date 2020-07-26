use crate::actors::messages::{ClientResponseMessage, RoomMessage};
use crate::actors::room::RoomActor;
use crate::data::UserData;
use actix::prelude::*;
use actix::Actor;
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};

/// Room manager. This is an actor that knows about all the created rooms and where each user is.
pub struct RoomManagerActor {
    rooms: HashMap<String, Addr<RoomActor>>,
    user_room_map: HashMap<String, HashSet<String>>,
}

impl Actor for RoomManagerActor {
    type Context = Context<Self>;
}

impl RoomManagerActor {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            user_room_map: HashMap::new(),
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
                ..
            } => {
                self.join_room(
                    room_name.to_owned(),
                    password.to_owned(),
                    user.user_id.to_owned(),
                    msg,
                );
            }
            RoomMessage::UserUpdated { user } => self.user_updated(user),
            _ => {}
        };
    }
}

impl RoomManagerActor {
    fn join_room(
        &mut self,
        room_name: String,
        password: String,
        user_id: String,
        msg: RoomMessage,
    ) {
        if !self.rooms.contains_key(&room_name) {
            self.create_room(room_name.clone(), password);
        }
        self.do_join_room(room_name, user_id, msg);
    }

    fn create_room(&mut self, room_name: String, password: String) {
        let room_actor = RoomActor::new(room_name.clone(), password).start();
        self.rooms.insert(room_name, room_actor);
    }

    fn do_join_room(&mut self, room_name: String, user_id: String, msg: RoomMessage) {
        match self.user_room_map.get_mut(&user_id) {
            None => println!("User trying to join not found in room manager."),
            Some(room_names) => {
                room_names.insert(room_name.to_owned());
                let room = self.rooms.get(&room_name).unwrap();
                room.do_send(msg);
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

    fn notify_rooms(&self, room_names: &HashSet<String>, msg: RoomMessage) {
        room_names
            .iter()
            .map(|room_name| self.rooms.get(room_name))
            .flatten()
            .for_each(|room| room.do_send(msg.clone()));
    }
}
