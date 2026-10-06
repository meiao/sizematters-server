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
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct Vote {
    pub value: Option<String>,
    pub has_voted: bool,
}

/// Reactive store for vote state per room, keyed by room_name -> user_id -> Vote.
#[derive(Clone, Copy)]
pub struct VoteStore {
    votes: RwSignal<HashMap<String, HashMap<String, Vote>>>,
    /// Spectator user ids per room. Spectators can't vote and are excluded
    /// from the "everyone voted" check.
    spectators: RwSignal<HashMap<String, HashSet<String>>>,
}

impl VoteStore {
    pub fn new() -> Self {
        Self {
            votes: RwSignal::new(HashMap::new()),
            spectators: RwSignal::new(HashMap::new()),
        }
    }

    pub fn votes_signal(&self) -> RwSignal<HashMap<String, HashMap<String, Vote>>> {
        self.votes
    }

    /// Initialize vote tracking when joining a room.
    /// Note: votes_cast is a count, we don't know *which* users voted yet.
    /// Everyone who joins a room starts as a spectator server-side, so we
    /// assume the same here; the next VoteStatus message will correct it.
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
        self.spectators.update(|spectators| {
            spectators.insert(room_name.to_string(), user_ids.iter().cloned().collect());
        });
    }

    /// Remove all vote data for a room when leaving it.
    pub fn leave_room(&self, room_name: &str) {
        self.votes.update(|votes| {
            votes.remove(room_name);
        });
        self.spectators.update(|spectators| {
            spectators.remove(room_name);
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
        // New joiners always start as spectators server-side.
        self.spectators.update(|spectators| {
            spectators
                .entry(room_name.to_string())
                .or_default()
                .insert(user_id.to_string());
        });
    }

    pub fn user_left(&self, room_name: &str, user_id: &str) {
        self.votes.update(|votes| {
            if let Some(room_votes) = votes.get_mut(room_name) {
                room_votes.remove(user_id);
            }
        });
        self.spectators.update(|spectators| {
            if let Some(room_spectators) = spectators.get_mut(room_name) {
                room_spectators.remove(user_id);
            }
        });
    }

    /// Replace the spectator set for a room with the authoritative list from VoteStatus.
    pub fn set_spectators(&self, room_name: &str, spectator_ids: &[String]) {
        self.spectators.update(|spectators| {
            spectators.insert(room_name.to_string(), spectator_ids.iter().cloned().collect());
        });
    }

    /// Correct spectator assumptions from a VoteResults message: anyone who cast
    /// a vote can't be a spectator, even if we hadn't seen a VoteStatus confirming it
    /// (e.g. we joined after they registered, and the round ended on the first VoteStatus
    /// we would have gotten, which VoteResults doesn't carry spectator info for).
    pub fn mark_as_voters(&self, room_name: &str, voter_ids: impl Iterator<Item = String>) {
        self.spectators.update(|spectators| {
            if let Some(room_spectators) = spectators.get_mut(room_name) {
                for voter_id in voter_ids {
                    room_spectators.remove(&voter_id);
                }
            }
        });
    }

    pub fn is_spectator(&self, room_name: &str, user_id: &str) -> bool {
        self.spectators.with(|spectators| {
            spectators
                .get(room_name)
                .map(|s| s.contains(user_id))
                .unwrap_or(false)
        })
    }

    pub fn spectator_count(&self, room_name: &str) -> usize {
        self.spectators
            .with(|spectators| spectators.get(room_name).map(|s| s.len()).unwrap_or(0))
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

    /// Check if all non-spectator users in a room have cast their votes.
    /// Uses `.with()` to avoid cloning the entire HashMap.
    pub fn is_voting_done(&self, room_name: &str) -> bool {
        let room_spectators = self
            .spectators
            .with(|spectators| spectators.get(room_name).cloned().unwrap_or_default());
        self.votes.with(|votes| {
            if let Some(room_votes) = votes.get(room_name) {
                let mut voters = room_votes
                    .iter()
                    .filter(|(uid, _)| !room_spectators.contains(*uid))
                    .peekable();
                voters.peek().is_some() && voters.all(|(_, v)| v.value.is_some())
            } else {
                false
            }
        })
    }
}
