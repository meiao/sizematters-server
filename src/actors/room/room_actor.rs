use crate::actors::messages::{ClientResponseMessage, RoomMessage};
use crate::data::UserData;
use actix::{Actor, Context, Handler, Recipient};
use std::borrow::Borrow;
use std::collections::HashMap;

pub struct RoomActor {
    name: String,
    password: String,
    user_map: HashMap<String, ConnectionInfo>,
    vote_map: HashMap<String, u64>,
}

impl RoomActor {
    pub fn new(name: String, password: String) -> RoomActor {
        let user_map = HashMap::new();
        let vote_map = HashMap::new();
        RoomActor {
            name,
            password,
            user_map,
            vote_map,
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
                user,
                recipient,
                ..
            } => self.join_room(password, user, recipient),
            RoomMessage::LeaveRoom { user_id, .. } => self.leave_room(user_id),
            RoomMessage::Vote { ref user_id, .. } => println!("vote received {}", user_id),
            RoomMessage::UserUpdated { user } => self.user_updated(user),
            _ => println!("Unsupported message reached RoomActor."),
        }
    }
}

impl RoomActor {
    fn join_room(
        &mut self,
        password: String,
        user: UserData,
        recipient: Recipient<ClientResponseMessage>,
    ) {
        let user_id = user.user_id.clone();

        if self.user_map.contains_key(&user_id) {
            let room_name = self.name.clone();
            recipient.do_send(ClientResponseMessage::AlreadyInRoom { room_name });
        } else if !(self.password.eq(&password)) {
            let room_name = self.name.clone();
            recipient.do_send(ClientResponseMessage::WrongPassword { room_name });
        } else {
            let user_entered_msg = ClientResponseMessage::UserEntered {
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
            let room_name = self.name.clone();
            let votes_cast = 0;
            let join_msg = ClientResponseMessage::RoomJoined {
                room_name,
                users,
                votes_cast,
            };
            joiner.do_send(join_msg);
        };
    }

    fn leave_room(&mut self, user_id: String) {
        let msg = ClientResponseMessage::UserLeft {
            user_id: user_id.clone(),
            room_name: self.name.clone(),
        };
        self.notify_users(msg);
        self.user_map.remove(&user_id);
    }

    fn user_updated(&mut self, user: UserData) {
        match self.user_map.get_mut(&user.user_id) {
            None => println!("Updating user not found in room."),
            Some(conn_info) => {
                conn_info.user = user.clone();
                self.notify_users(ClientResponseMessage::UserUpdated { user });
            }
        };
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

struct ConnectionInfo {
    user: UserData,
    recipient: Recipient<ClientResponseMessage>,
}
