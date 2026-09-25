/*
 * SizeMatters - a ticket sizing util
 * Copyright (C) 2026 Andre Onuki
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

use leptos::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Vote {
    pub value: Option<String>,
    pub has_voted: bool,
}

/// Reactive store for vote state per room, keyed by room_name -> user_id -> Vote.
#[derive(Clone, Copy)]
pub struct VoteStore {
    votes: RwSignal<HashMap<String, HashMap<String, Vote>>>,
}

impl VoteStore {
    pub fn new() -> Self {
        Self {
            votes: RwSignal::new(HashMap::new()),
        }
    }

    pub fn votes_signal(&self) -> RwSignal<HashMap<String, HashMap<String, Vote>>> {
        self.votes
    }

    /// Initialize vote tracking when joining a room.
    /// Note: votes_cast is a count, we don't know *which* users voted yet.
    /// All users start as not-voted; VoteStatus message will correct this.
    pub fn room_joined(&self, room_name: &str, user_ids: &[String], _votes_cast: usize) {
        self.votes.update(|votes| {
            let room_votes: HashMap<String, Vote> = user_ids
                .iter()
                .map(|uid| {
                    (
                        uid.clone(),
                        Vote {
                            value: None,
                            has_voted: false,
                        },
                    )
                })
                .collect();
            votes.insert(room_name.to_string(), room_votes);
        });
    }

    /// Remove all vote data for a room when leaving it.
    pub fn leave_room(&self, room_name: &str) {
        self.votes.update(|votes| {
            votes.remove(room_name);
        });
    }

    pub fn user_joined(&self, room_name: &str, user_id: &str) {
        self.votes.update(|votes| {
            if let Some(room_votes) = votes.get_mut(room_name) {
                room_votes.entry(user_id.to_string()).or_insert(Vote {
                    value: None,
                    has_voted: false,
                });
            }
        });
    }

    pub fn user_left(&self, room_name: &str, user_id: &str) {
        self.votes.update(|votes| {
            if let Some(room_votes) = votes.get_mut(room_name) {
                room_votes.remove(user_id);
            }
        });
    }

    pub fn own_vote(&self, room_name: &str, user_id: &str, size: String) {
        self.votes.update(|votes| {
            if let Some(room_votes) = votes.get_mut(room_name) {
                room_votes.insert(
                    user_id.to_string(),
                    Vote {
                        value: Some(size),
                        has_voted: true,
                    },
                );
            }
        });
    }

    /// Update who has voted (without revealing values).
    pub fn vote_status(&self, room_name: &str, status: &HashMap<String, bool>) {
        self.votes.update(|votes| {
            let room_votes = votes.entry(room_name.to_string()).or_default();
            for (user_id, has_voted) in status {
                let vote = room_votes.entry(user_id.clone()).or_insert(Vote {
                    value: None,
                    has_voted: false,
                });
                vote.has_voted = *has_voted;
            }
        });
    }

    /// Reveal all vote values when voting is complete.
    pub fn vote_results(&self, room_name: &str, results: &HashMap<String, String>) {
        self.votes.update(|votes| {
            let room_votes = votes.entry(room_name.to_string()).or_default();
            for (user_id, value) in results {
                room_votes.insert(
                    user_id.clone(),
                    Vote {
                        value: Some(value.clone()),
                        has_voted: true,
                    },
                );
            }
        });
    }

    pub fn new_vote(&self, room_name: &str) {
        self.votes.update(|votes| {
            if let Some(room_votes) = votes.get_mut(room_name) {
                for vote in room_votes.values_mut() {
                    vote.value = None;
                    vote.has_voted = false;
                }
            }
        });
    }

    /// Check if all users in a room have cast their votes.
    /// Uses `.with()` to avoid cloning the entire HashMap.
    pub fn is_voting_done(&self, room_name: &str) -> bool {
        self.votes.with(|votes| {
            if let Some(room_votes) = votes.get(room_name) {
                !room_votes.is_empty() && room_votes.values().all(|v| v.value.is_some())
            } else {
                false
            }
        })
    }
}
