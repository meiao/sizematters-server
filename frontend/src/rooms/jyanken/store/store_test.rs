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
use JyankenWeapon::{Paper, Rock, Scissor};

const ROOM: &str = "games";

fn ids(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

fn joined(user_ids: &[&str]) -> JyankenStore {
    let store = JyankenStore::new();
    store.room_joined(ROOM, &ids(user_ids));
    store
}

fn selected(entries: &[(&str, bool)]) -> HashMap<String, bool> {
    entries.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

#[test]
fn unknown_room_has_an_empty_state() {
    assert_eq!(JyankenStore::new().state("nope"), JyankenState::default());
}

#[test]
fn joining_tracks_everyone_as_a_spectator() {
    let store = joined(&["u1", "u2"]);

    let state = store.state(ROOM);
    assert_eq!(
        state.spectators,
        HashSet::from(["u1".to_string(), "u2".to_string()])
    );
    assert!(state.selected.is_empty());
    assert_eq!(state.own_weapon, None);
    assert_eq!(state.results, None);
}

#[test]
fn rejoining_resets_the_room() {
    let store = joined(&["u1"]);
    store.own_shoot(ROOM, Rock);
    store.game_results(
        ROOM,
        HashMap::new(),
        HashMap::from([("u1".to_string(), (1, 1))]),
    );

    store.room_joined(ROOM, &ids(&["u2"]));

    let state = store.state(ROOM);
    assert_eq!(state.spectators, HashSet::from(["u2".to_string()]));
    assert_eq!(state.own_weapon, None);
    assert_eq!(state.results, None);
    assert!(state.scoreboard.is_empty());
}

#[test]
fn user_joined_starts_as_a_spectator() {
    let store = joined(&["u1"]);
    store.user_joined(ROOM, "u2");
    assert!(store.state(ROOM).spectators.contains("u2"));
}

#[test]
fn user_left_is_forgotten() {
    let store = joined(&["u1", "u2"]);
    store.game_status(ROOM, selected(&[("u1", true)]), ids(&["u2"]));

    store.user_left(ROOM, "u1");
    store.user_left(ROOM, "u2");

    let state = store.state(ROOM);
    assert!(state.selected.is_empty());
    assert!(state.spectators.is_empty());
}

#[test]
fn own_shoot_records_our_weapon() {
    let store = joined(&["me"]);
    store.own_shoot(ROOM, Paper);
    assert_eq!(store.state(ROOM).own_weapon, Some(Paper));
}

#[test]
fn game_status_replaces_players_and_spectators() {
    let store = joined(&["u1", "u2"]);

    store.game_status(ROOM, selected(&[("u1", false)]), ids(&["u2"]));

    let state = store.state(ROOM);
    assert_eq!(state.selected, selected(&[("u1", false)]));
    assert_eq!(state.spectators, HashSet::from(["u2".to_string()]));
}

#[test]
fn game_status_keeps_our_weapon_while_someone_has_shot() {
    let store = joined(&["me", "u2"]);
    store.own_shoot(ROOM, Rock);

    store.game_status(ROOM, selected(&[("me", true), ("u2", false)]), vec![]);

    assert_eq!(store.state(ROOM).own_weapon, Some(Rock));
}

#[test]
fn game_status_where_nobody_shot_drops_our_weapon() {
    // e.g. we were moved to the spectators, which drops our pick server side
    let store = joined(&["me", "u2"]);
    store.own_shoot(ROOM, Rock);

    store.game_status(ROOM, selected(&[("u2", false)]), ids(&["me"]));

    assert_eq!(store.state(ROOM).own_weapon, None);
}

#[test]
fn game_status_after_results_keeps_our_weapon() {
    let store = joined(&["me"]);
    store.own_shoot(ROOM, Scissor);
    store.game_results(
        ROOM,
        HashMap::from([("me".to_string(), Scissor)]),
        HashMap::new(),
    );

    store.game_status(ROOM, selected(&[("me", false)]), vec![]);

    assert_eq!(store.state(ROOM).own_weapon, Some(Scissor));
}

#[test]
fn game_results_store_weapons_and_scoreboard() {
    let store = joined(&["u1", "u2"]);
    let weapons = HashMap::from([("u1".to_string(), Rock), ("u2".to_string(), Scissor)]);
    let scoreboard = HashMap::from([("u1".to_string(), (1, 3)), ("u2".to_string(), (0, 2))]);

    store.game_results(ROOM, weapons.clone(), scoreboard.clone());

    let state = store.state(ROOM);
    assert_eq!(state.results, Some(weapons));
    assert_eq!(state.scoreboard, scoreboard);
}

#[test]
fn new_game_resets_the_round_but_keeps_scores_and_players() {
    let store = joined(&["me", "u2"]);
    store.game_status(ROOM, selected(&[("me", true), ("u2", true)]), vec![]);
    store.own_shoot(ROOM, Rock);
    store.game_results(
        ROOM,
        HashMap::from([("me".to_string(), Rock)]),
        HashMap::from([("me".to_string(), (1, 1))]),
    );

    store.new_game(ROOM);

    let state = store.state(ROOM);
    assert_eq!(state.results, None);
    assert_eq!(state.own_weapon, None);
    assert_eq!(state.selected, selected(&[("me", false), ("u2", false)]));
    assert_eq!(
        state.scoreboard,
        HashMap::from([("me".to_string(), (1, 1))])
    );
}

#[test]
fn rooms_are_independent() {
    let store = joined(&["u1"]);
    store.room_joined("other", &ids(&["u1"]));

    store.own_shoot(ROOM, Rock);
    store.leave_room("other");

    assert_eq!(store.state(ROOM).own_weapon, Some(Rock));
    assert_eq!(store.state("other"), JyankenState::default());
}

#[test]
fn clear_forgets_every_room() {
    let store = joined(&["u1"]);
    store.own_shoot(ROOM, Rock);

    store.clear();

    assert_eq!(store.state(ROOM), JyankenState::default());
}
