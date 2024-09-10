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

use crate::actors::messages::ClientResponseMessage;
use crate::actors::room::RoomActor;
use std::collections::HashMap;

impl RoomActor {
    pub(super) fn vote(&mut self, user_id: String, size: String) {
        if self.voting_over() {
            match self.active_user_map.get(&user_id) {
                None => println!("RoomActor: User tried to cast vote in a room he is not in."),
                Some(user) => {
                    let msg = ClientResponseMessage::VotingOver;
                    self.notify_user(&user.user.user_id, &user.recipient, msg);
                }
            }
        } else {
            match self.active_user_map.get(&user_id) {
                None => println!("RoomActor: User tried to cast vote in a room he is not in."),
                Some(user) => {
                    let room_name = self.name.clone();
                    let msg = ClientResponseMessage::OwnVote { room_name: room_name,
                        size: size.clone() };
                    self.notify_user(&user.user.user_id, &user.recipient, msg);
                    let already_voted = self.vote_map.contains_key(&user_id);
                    self.vote_map.insert(user_id, size.clone());

                    if !already_voted {
                        self.send_vote_info();
                    }
                }
            }
        }
    }

    pub(super) fn send_vote_info(&self) {
        let room_name = self.name.clone();
        if self.voting_over() {
            let votes = self.vote_map.clone();
            let msg = ClientResponseMessage::VoteResults { room_name, votes };
            self.notify_users(msg);
        } else {
            let mut votes = HashMap::new();
            for user_id in self.active_user_map.keys() {
                let has_voted = self.vote_map.contains_key(user_id);
                votes.insert(user_id.to_owned(), has_voted);
            }
            let msg = ClientResponseMessage::VoteStatus { room_name, votes };
            self.notify_users(msg);
        }
    }

    pub(super) fn new_vote(&mut self, user_id: String) {
        if(self.in_room( &user_id))
        {
            self.voting_over = false;
            self.vote_map.clear();

            self.notify_users(ClientResponseMessage::NewVote {
                room_name: self.name.clone(),
            });
        }
        else
        {
            println!("RoomActor: User tried to request new vote in a room they is not in.");
        }
    }

    fn voting_over(&self) -> bool {
        self.vote_map.len() == self.active_user_map.len()
    }
}
