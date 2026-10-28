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
use sizematters_shared::avatar::AvatarStyle;

fn user(user_id: &str, name: &str) -> UserData {
    UserData {
        user_id: user_id.to_string(),
        name: name.to_string(),
        avatar: String::new(),
        avatar_style: AvatarStyle::default(),
    }
}

#[test]
fn starts_without_identity_or_users() {
    let store = UserStore::new();
    assert_eq!(store.own_user_id_untracked(), "");
    assert_eq!(store.own_user_id(), "");
    assert!(store.user_signal().get_untracked().is_empty());
}

#[test]
fn own_data_sets_identity_and_remembers_the_user() {
    let store = UserStore::new();

    store.own_data(user("me", "Ann"));

    assert_eq!(store.own_user_id_untracked(), "me");
    assert_eq!(store.own_user_id(), "me");
    assert_eq!(store.user_untracked("me"), Some(user("me", "Ann")));
}

#[test]
fn own_data_after_reconnect_switches_identity() {
    let store = UserStore::new();
    store.own_data(user("old", "Ann"));

    store.own_data(user("new", "Ann"));

    assert_eq!(store.own_user_id_untracked(), "new");
}

#[test]
fn user_updated_inserts_or_replaces() {
    let store = UserStore::new();

    store.user_updated(user("u1", "Bob"));
    assert_eq!(store.user_untracked("u1"), Some(user("u1", "Bob")));

    store.user_updated(user("u1", "Robert"));
    assert_eq!(store.user_untracked("u1"), Some(user("u1", "Robert")));
    assert_eq!(store.user_signal().get_untracked().len(), 1);
}

#[test]
fn room_joined_remembers_every_user() {
    let store = UserStore::new();
    store.user_updated(user("u1", "stale"));

    store.room_joined(&[user("u1", "Ann"), user("u2", "Bob")]);

    assert_eq!(store.user_untracked("u1"), Some(user("u1", "Ann")));
    assert_eq!(store.user_untracked("u2"), Some(user("u2", "Bob")));
}

#[test]
fn unknown_user_is_none() {
    assert_eq!(UserStore::new().user_untracked("ghost"), None);
}
