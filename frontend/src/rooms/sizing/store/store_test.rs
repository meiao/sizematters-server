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

const ROOM: &str = "team";

fn ids(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

fn votes(store: &VoteStore, room: &str) -> HashMap<String, Vote> {
    store
        .votes_signal()
        .get_untracked()
        .get(room)
        .cloned()
        .unwrap_or_default()
}

fn vote(store: &VoteStore, user_id: &str) -> Vote {
    votes(store, ROOM)
        .get(user_id)
        .cloned()
        .expect("user should have a vote entry")
}

fn none() -> Vote {
    Vote {
        value: None,
        has_voted: false,
    }
}

fn voted(value: Option<&str>) -> Vote {
    Vote {
        value: value.map(str::to_string),
        has_voted: true,
    }
}

fn joined(user_ids: &[&str]) -> VoteStore {
    let store = VoteStore::new();
    store.room_joined(ROOM, &ids(user_ids));
    store
}

#[test]
fn joining_tracks_everyone_as_a_spectator_without_votes() {
    let store = joined(&["u1", "u2"]);

    assert_eq!(votes(&store, ROOM).len(), 2);
    assert_eq!(vote(&store, "u1"), none());
    assert!(store.is_spectator(ROOM, "u1"));
    assert!(store.is_spectator(ROOM, "u2"));
    assert!(!store.is_revealed(ROOM));
    assert_eq!(store.voting_system(ROOM), VotingSystem::Fibonacci);
}

#[test]
fn rejoining_resets_the_room() {
    let store = joined(&["u1"]);
    store.vote_results(ROOM, &HashMap::from([("u1".to_string(), "3".to_string())]));

    store.room_joined(ROOM, &ids(&["u2"]));

    assert!(!votes(&store, ROOM).contains_key("u1"));
    assert_eq!(vote(&store, "u2"), none());
    assert!(!store.is_revealed(ROOM));
}

#[test]
fn unknown_rooms_have_defaults() {
    let store = VoteStore::new();
    assert!(!store.is_spectator("nope", "u1"));
    assert!(!store.is_revealed("nope"));
    assert_eq!(store.voting_system("nope"), VotingSystem::default());
}

#[test]
fn user_joined_starts_as_a_spectator() {
    let store = joined(&["u1"]);

    store.user_joined(ROOM, "u2");

    assert_eq!(vote(&store, "u2"), none());
    assert!(store.is_spectator(ROOM, "u2"));
}

#[test]
fn user_joined_keeps_an_existing_vote() {
    let store = joined(&["u1"]);
    store.own_vote(ROOM, "u1", "5".to_string());

    store.user_joined(ROOM, "u1");

    assert_eq!(vote(&store, "u1"), voted(Some("5")));
}

#[test]
fn user_left_forgets_vote_and_spectator_status() {
    let store = joined(&["u1", "u2"]);

    store.user_left(ROOM, "u2");

    assert!(!votes(&store, ROOM).contains_key("u2"));
    assert!(!store.is_spectator(ROOM, "u2"));
    assert!(store.is_spectator(ROOM, "u1"));
}

#[test]
fn set_spectators_replaces_the_set() {
    let store = joined(&["u1", "u2"]);

    store.set_spectators(ROOM, &ids(&["u2", "u3"]));

    assert!(!store.is_spectator(ROOM, "u1"));
    assert!(store.is_spectator(ROOM, "u2"));
    assert!(store.is_spectator(ROOM, "u3"));
}

#[test]
fn mark_as_voters_removes_them_from_the_spectators() {
    let store = joined(&["u1", "u2"]);

    store.mark_as_voters(ROOM, ids(&["u1", "ghost"]).into_iter());

    assert!(!store.is_spectator(ROOM, "u1"));
    assert!(store.is_spectator(ROOM, "u2"));
}

#[test]
fn own_vote_records_the_value() {
    let store = joined(&["u1"]);

    store.own_vote(ROOM, "u1", "8".to_string());

    assert_eq!(vote(&store, "u1"), voted(Some("8")));
}

#[test]
fn own_vote_in_an_unknown_room_is_ignored() {
    let store = VoteStore::new();
    store.own_vote("nope", "u1", "8".to_string());
    assert!(votes(&store, "nope").is_empty());
}

#[test]
fn vote_status_updates_flags_and_keeps_own_value() {
    let store = joined(&["u1", "u2"]);
    store.own_vote(ROOM, "u1", "3".to_string());

    store.vote_status(
        ROOM,
        &HashMap::from([
            ("u1".to_string(), true),
            ("u2".to_string(), true),
            ("u3".to_string(), false),
        ]),
    );

    assert_eq!(vote(&store, "u1"), voted(Some("3")));
    assert_eq!(vote(&store, "u2"), voted(None));
    assert_eq!(vote(&store, "u3"), none());
}

#[test]
fn vote_results_reveal_every_value() {
    let store = joined(&["u1", "u2"]);

    store.vote_results(
        ROOM,
        &HashMap::from([
            ("u1".to_string(), "S".to_string()),
            ("u2".to_string(), "XL".to_string()),
        ]),
    );

    assert_eq!(vote(&store, "u1"), voted(Some("S")));
    assert_eq!(vote(&store, "u2"), voted(Some("XL")));
    assert!(store.is_revealed(ROOM));
}

#[test]
fn new_vote_clears_votes_and_hides_results() {
    let store = joined(&["u1", "u2"]);
    store.vote_results(ROOM, &HashMap::from([("u1".to_string(), "1".to_string())]));

    store.new_vote(ROOM);

    assert_eq!(vote(&store, "u1"), none());
    assert_eq!(vote(&store, "u2"), none());
    assert!(!store.is_revealed(ROOM));
    // users are kept
    assert_eq!(votes(&store, ROOM).len(), 2);
}

#[test]
fn show_and_hide_results() {
    let store = joined(&["u1"]);
    store.show_results(ROOM);
    assert!(store.is_revealed(ROOM));
    store.hide_results(ROOM);
    assert!(!store.is_revealed(ROOM));
}

#[test]
fn voting_system_is_per_room() {
    let store = joined(&["u1"]);
    store.room_joined("other", &ids(&["u1"]));

    store.set_voting_system(ROOM, VotingSystem::TShirt);

    assert_eq!(store.voting_system(ROOM), VotingSystem::TShirt);
    assert_eq!(store.voting_system("other"), VotingSystem::Fibonacci);
}

#[test]
fn leave_room_forgets_everything_about_it() {
    let store = joined(&["u1"]);
    store.room_joined("other", &ids(&["u1"]));
    store.set_voting_system(ROOM, VotingSystem::TShirt);
    store.show_results(ROOM);

    store.leave_room(ROOM);

    assert!(!store.votes_signal().get_untracked().contains_key(ROOM));
    assert!(!store.is_spectator(ROOM, "u1"));
    assert!(!store.is_revealed(ROOM));
    assert_eq!(store.voting_system(ROOM), VotingSystem::default());
    assert!(store.is_spectator("other", "u1"));
}

#[test]
fn clear_forgets_every_room() {
    let store = joined(&["u1"]);
    store.room_joined("other", &ids(&["u1"]));
    store.set_voting_system("other", VotingSystem::FistToFive);

    store.clear();

    assert!(store.votes_signal().get_untracked().is_empty());
    assert!(!store.is_spectator("other", "u1"));
    assert_eq!(store.voting_system("other"), VotingSystem::default());
}
