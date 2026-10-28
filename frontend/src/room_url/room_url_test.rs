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

fn url(room_type: Option<RoomType>, name: &str, pw: &str) -> Option<RoomUrl> {
    Some(RoomUrl {
        room_name: name.to_string(),
        hashed_password: pw.to_string(),
        room_type,
    })
}

#[test]
fn parses_full_url_with_room_type() {
    assert_eq!(
        parse_room_url("https://sizematters.dev/room/jyanken/team-a/abc123"),
        url(Some(RoomType::Jyanken), "team-a", "abc123")
    );
}

#[test]
fn parses_legacy_url_without_room_type() {
    assert_eq!(
        parse_room_url("http://localhost:8080/room/team-a/abc123"),
        url(None, "team-a", "abc123")
    );
}

#[test]
fn unknown_room_type_falls_back_to_join() {
    assert_eq!(
        parse_room_url("https://sizematters.dev/room/chess/team-a/abc123"),
        url(None, "team-a", "abc123")
    );
}

#[test]
fn room_type_slug_is_case_insensitive() {
    assert_eq!(
        parse_room_url("/room/SIZING/team-a/abc"),
        url(Some(RoomType::Sizing), "team-a", "abc")
    );
}

#[test]
fn accepts_bare_path_whitespace_trailing_slash_query_and_fragment() {
    assert_eq!(
        parse_room_url("  /room/sizing/team-a/abc123/?x=1#top \n"),
        url(Some(RoomType::Sizing), "team-a", "abc123")
    );
}

#[test]
fn fragment_before_query_is_dropped_too() {
    assert_eq!(
        parse_room_url("/room/team-a/abc#frag?x=1"),
        url(None, "team-a", "abc")
    );
}

#[test]
fn empty_segments_are_skipped() {
    assert_eq!(
        parse_room_url("https://sizematters.dev/room//sizing//team-a/abc"),
        url(Some(RoomType::Sizing), "team-a", "abc")
    );
}

#[test]
fn path_prefix_before_room_is_ignored() {
    assert_eq!(
        parse_room_url("https://example.com/apps/sizematters/room/team-a/abc"),
        url(None, "team-a", "abc")
    );
}

#[test]
fn first_room_segment_wins() {
    // A room literally named "room" still parses.
    assert_eq!(
        parse_room_url("/room/sizing/room/abc"),
        url(Some(RoomType::Sizing), "room", "abc")
    );
}

#[test]
fn decodes_percent_escapes() {
    assert_eq!(
        parse_room_url("https://sizematters.dev/room/sizing/my%20room%C3%A9/abc"),
        url(Some(RoomType::Sizing), "my roomé", "abc")
    );
}

#[test]
fn decodes_escapes_in_the_password_too() {
    assert_eq!(parse_room_url("/room/team/a%2Fb"), url(None, "team", "a/b"));
}

#[test]
fn rejects_non_room_urls() {
    assert_eq!(parse_room_url(""), None);
    assert_eq!(parse_room_url("team-a"), None);
    assert_eq!(parse_room_url("https://sizematters.dev/main"), None);
    assert_eq!(parse_room_url("https://sizematters.dev/room/team-a"), None);
    assert_eq!(parse_room_url("https://sizematters.dev/room/a/b/c/d"), None);
    assert_eq!(
        parse_room_url("https://sizematters.dev/room/sizing/bad%zz/abc"),
        None
    );
}

#[test]
fn rejects_segments_that_decode_to_nothing_valid() {
    // truncated escape
    assert_eq!(parse_room_url("/room/team/abc%4"), None);
    // invalid UTF-8
    assert_eq!(parse_room_url("/room/team%FF/abc"), None);
}

#[test]
fn percent_decode_handles_plain_and_escaped_text() {
    assert_eq!(percent_decode("plain").as_deref(), Some("plain"));
    assert_eq!(percent_decode("").as_deref(), Some(""));
    assert_eq!(percent_decode("%41%62c").as_deref(), Some("Abc"));
    assert_eq!(percent_decode("100%25").as_deref(), Some("100%"));
    assert_eq!(percent_decode("%e2%9c%8a").as_deref(), Some("✊"));
}

#[test]
fn percent_decode_rejects_malformed_escapes() {
    assert_eq!(percent_decode("%"), None);
    assert_eq!(percent_decode("%4"), None);
    assert_eq!(percent_decode("%G0"), None);
    // a multi-byte char right after % must not panic
    assert_eq!(percent_decode("%é"), None);
}
