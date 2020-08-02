use crate::actors::messages::{ClientResponseMessage, RoomMessage};
use crate::data::UserData;
use actix::{Actor, ActorContext, Context, Handler, Recipient};
use std::borrow::Borrow;
use std::collections::HashMap;

pub struct RoomActor {
    name: String,
    password: String,
    user_map: HashMap<String, ConnectionInfo>,
    vote_map: HashMap<String, u64>,
    room_manager: Recipient<RoomMessage>,
    voting_over: bool,
}

impl RoomActor {
    pub fn new(name: String, password: String, room_manager: Recipient<RoomMessage>) -> RoomActor {
        RoomActor {
            name,
            password,
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
                user,
                recipient,
                ..
            } => self.join_room(password, user, recipient),
            RoomMessage::LeaveRoom { user_id, .. } => self.leave_room(user_id, ctx),
            RoomMessage::Vote { user_id, size, .. } => self.vote(user_id, size),
            RoomMessage::NewVote { user_id, .. } => self.new_vote(user_id),
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
            let join_msg = ClientResponseMessage::RoomJoined {
                room_name: self.name.clone(),
                users,
                votes_cast: self.vote_map.len(),
            };
            joiner.do_send(join_msg);
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

        let msg = ClientResponseMessage::VotesCast {
            votes_cast: self.vote_map.len(),
            room_name: self.name.clone(),
        };
        self.notify_users(msg);

        if (self.user_map.is_empty()) {
            let msg = RoomMessage::RoomClosing {
                room_name: self.name.clone(),
            };
            self.room_manager.borrow().do_send(msg);
            ctx.stop();
        }
    }

    fn vote(&mut self, user_id: String, size: u64) {
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
                    let msg = ClientResponseMessage::UserVote { room_name, size };
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

    fn new_vote(&mut self, user_id: String) {
        if !self.user_map.contains_key(&user_id) {
            println!("User tried to request new vote in a room they is not in.");
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
