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

fn rooms(store: &RoomStore) -> Vec<RoomStatus> {
    store.rooms_signal().get_untracked()
}

fn room(store: &RoomStore, room_name: &str) -> RoomStatus {
    rooms(store)
        .into_iter()
        .find(|r| r.room_name == room_name)
        .expect("room should be known")
}

fn store_with(room_names: &[&str]) -> RoomStore {
    let store = RoomStore::new();
    for name in room_names {
        store.room_joined(
            name.to_string(),
            format!("hash of {name}"),
            RoomType::Sizing,
            vec![user("u1", "Ann")],
        );
    }
    store
}

#[test]
fn starts_empty() {
    assert!(rooms(&RoomStore::new()).is_empty());
}

#[test]
fn joined_rooms_are_listed_in_join_order() {
    let store = store_with(&["b", "a", "c"]);
    let names: Vec<String> = rooms(&store).into_iter().map(|r| r.room_name).collect();
    assert_eq!(names, vec!["b", "a", "c"]);
    let a = room(&store, "a");
    assert_eq!(a.hashed_password, "hash of a");
    assert_eq!(a.room_type, RoomType::Sizing);
    assert_eq!(a.users, vec![user("u1", "Ann")]);
    assert_eq!(a.selected_user, None);
}

#[test]
fn rejoining_replaces_the_room_in_place() {
    let store = store_with(&["a", "b"]);
    store.randomized("a", "u1");

    store.room_joined(
        "a".to_string(),
        "new hash".to_string(),
        RoomType::Jyanken,
        vec![user("u9", "Zed")],
    );

    let all = rooms(&store);
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].room_name, "a");
    assert_eq!(all[0].hashed_password, "new hash");
    assert_eq!(all[0].room_type, RoomType::Jyanken);
    assert_eq!(all[0].users, vec![user("u9", "Zed")]);
    assert_eq!(all[0].selected_user, None);
}

#[test]
fn clear_forgets_every_room() {
    let store = store_with(&["a", "b"]);
    store.clear();
    assert!(rooms(&store).is_empty());
}

#[test]
fn user_joined_adds_the_user_once() {
    let store = store_with(&["a"]);

    store.user_joined("a", user("u2", "Bob"));
    store.user_joined("a", user("u2", "Bob again"));

    let users = room(&store, "a").users;
    assert_eq!(users, vec![user("u1", "Ann"), user("u2", "Bob")]);
}

#[test]
fn user_joined_for_an_unknown_room_is_ignored() {
    let store = store_with(&["a"]);
    store.user_joined("zzz", user("u2", "Bob"));
    assert_eq!(rooms(&store).len(), 1);
    assert_eq!(room(&store, "a").users.len(), 1);
}

#[test]
fn user_updated_replaces_the_user_in_every_room() {
    let store = store_with(&["a", "b"]);
    store.user_joined("a", user("u2", "Bob"));

    store.user_updated(&user("u1", "Annie"));

    for name in ["a", "b"] {
        let users = room(&store, name).users;
        assert!(users.contains(&user("u1", "Annie")), "{name}");
        assert!(!users.iter().any(|u| u.name == "Ann"), "{name}");
    }
    assert!(room(&store, "a").users.contains(&user("u2", "Bob")));
}

#[test]
fn user_updated_does_not_add_unknown_users() {
    let store = store_with(&["a"]);
    store.user_updated(&user("u2", "Bob"));
    assert_eq!(room(&store, "a").users, vec![user("u1", "Ann")]);
}

#[test]
fn user_left_removes_the_user_from_that_room_only() {
    let store = store_with(&["a", "b"]);

    store.user_left("a", "u1");

    assert!(room(&store, "a").users.is_empty());
    assert_eq!(room(&store, "b").users.len(), 1);
}

#[test]
fn room_type_untracked() {
    let store = RoomStore::new();
    store.room_joined("j".to_string(), String::new(), RoomType::Jyanken, vec![]);
    assert_eq!(store.room_type_untracked("j"), Some(RoomType::Jyanken));
    assert_eq!(store.room_type_untracked("nope"), None);
}

#[test]
fn randomized_records_and_clear_resets_the_selection() {
    let store = store_with(&["a", "b"]);

    store.randomized("a", "u1");
    assert_eq!(room(&store, "a").selected_user.as_deref(), Some("u1"));
    assert_eq!(room(&store, "b").selected_user, None);

    store.clear_selected_user("a");
    assert_eq!(room(&store, "a").selected_user, None);
}

#[test]
fn leave_room_removes_only_that_room() {
    let store = store_with(&["a", "b", "c"]);
    store.leave_room("b");
    let names: Vec<String> = rooms(&store).into_iter().map(|r| r.room_name).collect();
    assert_eq!(names, vec!["a", "c"]);
}
