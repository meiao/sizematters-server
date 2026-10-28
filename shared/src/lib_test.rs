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
use std::collections::HashSet;

#[test]
fn url_slug_round_trips_for_every_room_type() {
    for room_type in RoomType::iter() {
        assert_eq!(
            RoomType::from_url_slug(room_type.url_slug()),
            Some(room_type)
        );
    }
}

#[test]
fn from_url_slug_ignores_case() {
    assert_eq!(RoomType::from_url_slug("Jyanken"), Some(RoomType::Jyanken));
    assert_eq!(RoomType::from_url_slug("SIZING"), Some(RoomType::Sizing));
}

#[test]
fn from_url_slug_rejects_unknown_types() {
    assert_eq!(RoomType::from_url_slug("chess"), None);
    assert_eq!(RoomType::from_url_slug(""), None);
    assert_eq!(RoomType::from_url_slug(" sizing"), None);
}

#[test]
fn url_slugs_are_unique_and_url_safe() {
    let slugs: HashSet<&str> = RoomType::iter().map(|t| t.url_slug()).collect();
    assert_eq!(slugs.len(), RoomType::iter().count());
    for slug in slugs {
        assert!(!slug.is_empty());
        assert!(
            slug.chars().all(|c| c.is_ascii_lowercase()),
            "slug {slug:?} should be lowercase ascii"
        );
    }
}

#[test]
fn display_names_are_unique_and_non_empty() {
    let names: HashSet<&str> = RoomType::iter().map(|t| t.display_name()).collect();
    assert_eq!(names.len(), RoomType::iter().count());
    assert!(names.iter().all(|n| !n.is_empty()));
}

#[test]
fn room_type_serializes_as_variant_name() {
    assert_eq!(
        serde_json::to_string(&RoomType::Sizing).unwrap(),
        "\"Sizing\""
    );
    assert_eq!(
        serde_json::from_str::<RoomType>("\"Jyanken\"").unwrap(),
        RoomType::Jyanken
    );
}

#[test]
fn new_user_gets_default_name_and_style() {
    let user = UserData::new("id-1".to_string(), "data:avatar".to_string());
    assert_eq!(user.user_id, "id-1");
    assert_eq!(user.avatar, "data:avatar");
    assert_eq!(user.name, "Shirtless Muppet");
    assert_eq!(user.avatar_style, AvatarStyle::default());
}

#[test]
fn set_avatar_replaces_only_the_avatar() {
    let mut user = UserData::new("id-1".to_string(), "old".to_string());
    let before = user.clone();
    user.set_avatar("new");
    assert_eq!(user.avatar, "new");
    assert_eq!(user.user_id, before.user_id);
    assert_eq!(user.name, before.name);
    assert_eq!(user.avatar_style, before.avatar_style);
}

#[test]
fn user_data_round_trips_through_json() {
    let mut user = UserData::new("id-1".to_string(), "a".to_string());
    user.name = "Émile".to_string();
    user.avatar_style = AvatarStyle::Waves;
    let json = serde_json::to_string(&user).unwrap();
    assert_eq!(serde_json::from_str::<UserData>(&json).unwrap(), user);
}
