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
use strum::IntoEnumIterator;

#[test]
fn default_style_is_identicon() {
    assert_eq!(AvatarStyle::default(), AvatarStyle::Identicon);
}

#[test]
fn every_style_round_trips_through_json() {
    for style in AvatarStyle::iter() {
        let json = serde_json::to_string(&style).unwrap();
        assert_eq!(serde_json::from_str::<AvatarStyle>(&json).unwrap(), style);
    }
}

#[test]
fn style_serializes_as_variant_name() {
    assert_eq!(
        serde_json::to_string(&AvatarStyle::Pixelbot).unwrap(),
        "\"Pixelbot\""
    );
}

#[test]
fn unknown_style_is_rejected() {
    assert!(serde_json::from_str::<AvatarStyle>("\"Bottts\"").is_err());
}
