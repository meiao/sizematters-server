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

use crate::actors::rooms::room::{RoomBehavior, RoomState};
use rand::RngExt;
use sizematters_shared::messages::{ClientRequestMessage, ClientResponseMessage, SizingMessage};
use std::collections::HashMap;
use std::sync::Arc;

pub(super) struct SizingBehavior {
    vote_map: HashMap<Arc<String>, String>,
}

impl SizingBehavior {
    pub(super) fn new() -> Self {
        SizingBehavior {
            vote_map: HashMap::new(),
        }
    }
}

impl RoomBehavior<SizingMessage> for SizingBehavior {
    fn on_join_room(&mut self, user_id: &Arc<String>, room: &mut RoomState) {
        let join_msg = ClientResponseMessage::RoomJoined {
            room_name: room.room_name(),
            hashed_password: room.hashed_password(),
            users: room.users(),
            votes_cast: self.vote_map.len(),
        };
        room.notify_user(user_id, join_msg);
        room.notify_user(user_id, self.vote_status(room.room_name(), room));
    }

    fn on_leave_room(&mut self, user_id: &Arc<String>, room: &mut RoomState) {
        self.vote_map.remove(user_id);
        self.send_vote_info(room);
    }

    fn on_specific_message(&mut self, user_id: Arc<String>, msg: SizingMessage, room: &mut RoomState) {
        match msg {
            SizingMessage::Vote { size } => self.vote(user_id, size, room),
            SizingMessage::NewVote => self.new_vote(&user_id, room),
            SizingMessage::Randomize => self.randomize(room),
            SizingMessage::RegisterToVote => self.register_to_vote(user_id, room),
            SizingMessage::RevokeVotingRights { target_id } => self.revoke_voting_rights(user_id, target_id, room),
        }
    }
}

impl SizingBehavior {
    fn vote(&mut self, user_id: Arc<String>, size: String, room: &RoomState) {
        if !room.contains_user(&user_id) {
            println!("RoomActor: User tried to cast vote in a room he is not in.");
            return;
        }

        if self.voting_over(room) {
            room.notify_user(&user_id, ClientResponseMessage::VotingOver);
            return;
        }

        if room.is_spectator(&user_id) {
            room.notify_user(&user_id, ClientResponseMessage::Error { msg: "You must register to vote to be able to vote".to_string() });
            return;
        }

        let msg = ClientResponseMessage::OwnVote {
            room_name: room.room_name(),
            size: size.clone(),
        };
        room.notify_user(&user_id, msg);

        let already_voted = self.vote_map.insert(user_id, size).is_some();
        if !already_voted {
            self.send_vote_info(room);
        }
    }

    fn new_vote(&mut self, user_id: &Arc<String>, room: &RoomState) {
        if !room.contains_user(user_id) {
            println!("RoomActor: User tried to request new vote in a room they is not in.");
            return;
        }

        self.vote_map.clear();

        room.notify_users(ClientResponseMessage::NewVote {
            room_name: room.room_name(),
        });
    }

    fn randomize(&self, room: &RoomState) {
        let users: Vec<&Arc<String>> = room.user_ids().collect();
        if users.is_empty() {
            println!("RoomActor: User not found in room.");
            return;
        }

        let selected_user = users[rand::rng().random_range(0..users.len())];
        room.notify_users(ClientResponseMessage::Randomized {
            room_name: room.room_name(),
            selected_user_id: (**selected_user).clone(),
        });
    }

    fn register_to_vote(&self, user_id: Arc<String>, room: &mut RoomState) {
        room.remove_spectator(user_id);
        self.send_vote_info(room)
    }

    fn revoke_voting_rights(&self, user_id: Arc<String>, target_id: String, room: &mut RoomState) {
        let target_id = Arc::new(target_id);
        if !room.contains_user(&target_id) {
            room.notify_user(&user_id, ClientResponseMessage::Error { msg: "User not found.".to_string() });
            return;
        }
        if room.is_spectator(&target_id) {
            return;
        }
        room.add_spectator(target_id);
        self.send_vote_info(room);
    }

    fn send_vote_info(&self, room: &RoomState) {
        let room_name = room.room_name();
        if self.voting_over(room) {
            let votes: HashMap<String, String> = self
                .vote_map
                .iter()
                .map(|(user_id, size)| ((**user_id).clone(), size.clone()))
                .collect();
            room.notify_users(ClientResponseMessage::VoteResults { room_name, votes });
        } else {
            room.notify_users(self.vote_status(room_name, room));
        }
    }

    fn vote_status(&self, room_name: String, room: &RoomState) -> ClientResponseMessage {
        let votes: HashMap<String, bool> = room
            .user_ids()
            .filter(|user_id| !room.is_spectator(&user_id))
            .map(|user_id| ((**user_id).clone(), self.vote_map.contains_key(user_id)))
            .collect();
        let spectators = room.spectators();
        ClientResponseMessage::VoteStatus { room_name, votes, spectators }
    }

    fn voting_over(&self, room: &RoomState) -> bool {
        self.vote_map.len() == room.user_count() - room.spectator_count()
    }
}
