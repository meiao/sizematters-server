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
use sizematters_shared::messages::{ClientResponseMessage, SizingMessage};
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
    fn on_join_room(&mut self, user_id: &Arc<String>, room: &RoomState) {
        let join_msg = ClientResponseMessage::RoomJoined {
            room_name: room.room_name(),
            hashed_password: room.hashed_password(),
            users: room.users(),
            votes_cast: self.vote_map.len(),
        };
        room.notify_user(user_id, join_msg);
    }

    fn on_leave_room(&mut self, user_id: &Arc<String>, room: &RoomState) {
        self.vote_map.remove(user_id);
        self.send_vote_info(room);
    }

    fn on_specific_message(&mut self, user_id: Arc<String>, msg: SizingMessage, room: &RoomState) {
        match msg {
            SizingMessage::Vote { size } => self.vote(user_id, size, room),
            SizingMessage::NewVote => self.new_vote(&user_id, room),
            SizingMessage::Randomize => self.randomize(room),
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
            let votes: HashMap<String, bool> = room
                .user_ids()
                .map(|user_id| ((**user_id).clone(), self.vote_map.contains_key(user_id)))
                .collect();
            room.notify_users(ClientResponseMessage::VoteStatus { room_name, votes });
        }
    }

    fn voting_over(&self, room: &RoomState) -> bool {
        self.vote_map.len() == room.user_count()
    }
}
