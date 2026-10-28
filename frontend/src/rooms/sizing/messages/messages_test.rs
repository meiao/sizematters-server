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

use super::*;
use crate::stores::{RoomStore, UserStore};
use sizematters_shared::avatar::AvatarStyle;
use sizematters_shared::messages::sizing::VotingSystem;
use sizematters_shared::{RoomType, UserData};
use std::collections::HashMap;

const ROOM: &str = "team";

struct Fixture {
    votes: VoteStore,
    ctx: HandlerCtx,
}

impl Fixture {
    /// We are "me", in ROOM with "u2", and the randomizer picked u2.
    fn new() -> Self {
        let user_store = UserStore::new();
        user_store.own_data(UserData {
            user_id: "me".to_string(),
            name: "Me".to_string(),
            avatar: String::new(),
            avatar_style: AvatarStyle::default(),
        });
        let room_store = RoomStore::new();
        room_store.room_joined(ROOM.to_string(), String::new(), RoomType::Sizing, vec![]);
        room_store.randomized(ROOM, "u2");
        let votes = VoteStore::new();
        votes.room_joined(ROOM, &["me".to_string(), "u2".to_string()]);
        Fixture {
            votes,
            ctx: HandlerCtx {
                room_store,
                user_store,
                last_error: RwSignal::new(None),
            },
        }
    }

    fn handle(&self, msg: SizingResponseMessage) {
        let payload = serde_json::to_string(&msg).unwrap();
        handle_message(&payload, ROOM, self.votes, self.ctx);
    }

    fn vote_of(&self, user_id: &str) -> Option<(Option<String>, bool)> {
        self.votes
            .votes_signal()
            .get_untracked()
            .get(ROOM)?
            .get(user_id)
            .map(|v| (v.value.clone(), v.has_voted))
    }

    fn selected_user(&self) -> Option<String> {
        self.ctx.room_store.rooms_signal().get_untracked()[0]
            .selected_user
            .clone()
    }
}

#[test]
fn own_vote_is_recorded_for_ourselves() {
    let fixture = Fixture::new();

    fixture.handle(SizingResponseMessage::OwnVote {
        size: "5".to_string(),
    });

    assert_eq!(fixture.vote_of("me"), Some((Some("5".to_string()), true)));
    assert_eq!(fixture.vote_of("u2"), Some((None, false)));
}

#[test]
fn vote_status_updates_system_flags_and_spectators() {
    let fixture = Fixture::new();

    fixture.handle(SizingResponseMessage::VoteStatus {
        votes: HashMap::from([("me".to_string(), false), ("u2".to_string(), true)]),
        voting_system: VotingSystem::FistToFive,
        spectators: vec![],
    });

    assert_eq!(fixture.votes.voting_system(ROOM), VotingSystem::FistToFive);
    assert_eq!(fixture.vote_of("u2"), Some((None, true)));
    assert!(!fixture.votes.is_spectator(ROOM, "me"));
    assert!(!fixture.votes.is_spectator(ROOM, "u2"));
}

#[test]
fn vote_results_reveal_values_and_mark_voters() {
    let fixture = Fixture::new();

    fixture.handle(SizingResponseMessage::VoteResults {
        votes: HashMap::from([("u2".to_string(), "13".to_string())]),
    });

    assert!(fixture.votes.is_revealed(ROOM));
    assert_eq!(fixture.vote_of("u2"), Some((Some("13".to_string()), true)));
    assert!(!fixture.votes.is_spectator(ROOM, "u2"));
    assert!(fixture.votes.is_spectator(ROOM, "me"));
}

#[test]
fn new_vote_resets_votes_and_the_randomizer_pick() {
    let fixture = Fixture::new();
    fixture.handle(SizingResponseMessage::VoteResults {
        votes: HashMap::from([("u2".to_string(), "13".to_string())]),
    });

    fixture.handle(SizingResponseMessage::NewVote);

    assert!(!fixture.votes.is_revealed(ROOM));
    assert_eq!(fixture.vote_of("u2"), Some((None, false)));
    assert_eq!(fixture.selected_user(), None);
}

#[test]
fn voting_system_change_also_starts_a_new_vote() {
    let fixture = Fixture::new();
    fixture.handle(SizingResponseMessage::OwnVote {
        size: "5".to_string(),
    });

    fixture.handle(SizingResponseMessage::VotingSystemChanged {
        new_system: VotingSystem::TShirt,
    });

    assert_eq!(fixture.votes.voting_system(ROOM), VotingSystem::TShirt);
    assert_eq!(fixture.vote_of("me"), Some((None, false)));
    assert_eq!(fixture.selected_user(), None);
}

#[test]
fn voting_over_is_shown_as_an_error() {
    let fixture = Fixture::new();

    fixture.handle(SizingResponseMessage::VotingOver);

    assert_eq!(
        fixture.ctx.last_error.get_untracked().as_deref(),
        Some("Voting is already over.")
    );
}

#[test]
fn unreadable_payload_is_ignored() {
    let fixture = Fixture::new();

    handle_message(r#"{"type":"Nope"}"#, ROOM, fixture.votes, fixture.ctx);

    assert_eq!(fixture.ctx.last_error.get_untracked(), None);
    assert_eq!(fixture.vote_of("me"), Some((None, false)));
    assert_eq!(fixture.selected_user().as_deref(), Some("u2"));
}
