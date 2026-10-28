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
use strum::IntoEnumIterator;

const SVG_DATA_URI: &str = "data:image/svg+xml";

#[test]
fn default_avatar_is_an_svg_data_uri() {
    assert!(default_avatar("user-1").starts_with(SVG_DATA_URI));
}

#[test]
fn default_avatar_is_deterministic_per_seed() {
    assert_eq!(default_avatar("user-1"), default_avatar("user-1"));
    assert_ne!(default_avatar("user-1"), default_avatar("user-2"));
}

#[test]
fn default_avatar_matches_the_identicon_style() {
    assert_eq!(
        default_avatar("user-1"),
        styled_avatar("user-1", &AvatarStyle::Identicon)
    );
}

#[test]
fn every_style_renders_a_distinct_avatar() {
    let avatars: HashSet<String> = AvatarStyle::iter()
        .map(|style| {
            let avatar = styled_avatar("Ann", &style);
            assert!(avatar.starts_with(SVG_DATA_URI), "{style:?}");
            avatar
        })
        .collect();
    assert_eq!(avatars.len(), AvatarStyle::iter().count());
}

#[test]
fn styled_avatar_is_deterministic_per_seed() {
    for style in AvatarStyle::iter() {
        assert_eq!(
            styled_avatar("Ann", &style),
            styled_avatar("Ann", &style),
            "{style:?}"
        );
        assert_ne!(
            styled_avatar("Ann", &style),
            styled_avatar("Bob", &style),
            "{style:?}"
        );
    }
}

#[test]
fn styled_avatar_accepts_any_seed() {
    for seed in ["", " ", "Émile 🎉", &"x".repeat(1000)] {
        assert!(styled_avatar(seed, &AvatarStyle::Waves).starts_with(SVG_DATA_URI));
    }
}
