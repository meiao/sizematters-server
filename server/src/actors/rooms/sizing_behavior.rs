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
    voting_over: bool,
}

impl SizingBehavior {
    pub(super) fn new() -> Self {
        SizingBehavior {
            vote_map: HashMap::new(),
            voting_over: false,
        }
    }
}

impl RoomBehavior<SizingMessage> for SizingBehavior {
    fn on_join_room(&mut self, user_id: &Arc<String>, room: &mut RoomState) {
        room.notify_user(user_id, self.vote_status(room));
        if self.voting_over {
            room.notify_user(user_id, self.vote_results(room))
        }
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
    fn vote(&mut self, user_id: Arc<String>, size: String, room: &mut RoomState) {
        if !room.contains_user(&user_id) {
            println!("RoomActor: User tried to cast vote in a room he is not in.");
            return;
        }

        if self.voting_over {
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
        self.voting_over = false;

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

    fn register_to_vote(&mut self, user_id: Arc<String>, room: &mut RoomState) {
        room.remove_spectator(user_id);
        self.send_vote_info(room)
    }

    fn revoke_voting_rights(&mut self, user_id: Arc<String>, target_id: String, room: &mut RoomState) {
        let target_id = Arc::new(target_id);
        if !room.contains_user(&target_id) {
            room.notify_user(&user_id, ClientResponseMessage::Error { msg: "User not found.".to_string() });
            return;
        }
        if room.is_spectator(&target_id) {
            return;
        }
        self.vote_map.remove(&target_id);
        room.add_spectator(target_id);
        self.send_vote_info(room);
    }

    fn send_vote_info(&mut self, room: &mut RoomState) {
        room.notify_users(self.vote_status(room));
        if self.all_voted(room) {
            self.voting_over = true;
        }
        if self.voting_over {
            room.notify_users(self.vote_results(room));
        }
    }

    fn vote_status(&self, room: &RoomState) -> ClientResponseMessage {
        let room_name = room.room_name();
        let votes: HashMap<String, bool> = room
            .user_ids()
            .filter(|user_id| !room.is_spectator(&user_id))
            .map(|user_id| ((**user_id).clone(), self.vote_map.contains_key(user_id)))
            .collect();
        let spectators = room.spectators();
        ClientResponseMessage::VoteStatus { room_name, votes, spectators }
    }

    fn vote_results(&self, room: &RoomState) -> ClientResponseMessage {
        let room_name = room.room_name();
        let votes: HashMap<String, String> = self
            .vote_map
            .iter()
            .map(|(user_id, size)| ((**user_id).clone(), size.clone()))
            .collect();
        ClientResponseMessage::VoteResults { room_name, votes }
    }

    fn all_voted(&self, room: &RoomState) -> bool {
        let active_user_count = room.user_count() - room.spectator_count();
        active_user_count > 0 && self.vote_map.len() == active_user_count
    }
}
