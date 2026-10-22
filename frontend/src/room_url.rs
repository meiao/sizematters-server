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

//! Parsing of shareable room URLs pasted by the user, so they can create/join a room
//! without reloading the page on that URL.
//!
//! Accepted shapes (the same ones the router handles, see `ConnectingPage`):
//! - `.../room/{room_type}/{room_name}/{hashed_password}` creates or joins the room;
//! - `.../room/{room_name}/{hashed_password}` (legacy) joins an existing room.
//!
//! Anything before `/room/` (scheme, host, port) is ignored, as is any query string or
//! fragment, so a full URL or a bare path both work.

use sizematters_shared::RoomType;

/// A room reference parsed from a shareable URL. The password is already hashed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomUrl {
    pub room_name: String,
    pub hashed_password: String,
    /// `None` for legacy URLs, or when the type is unknown: the room can only be joined.
    pub room_type: Option<RoomType>,
}

/// Parses a pasted room URL, or returns `None` if it isn't one.
pub fn parse_room_url(input: &str) -> Option<RoomUrl> {
    let input = input.trim();
    // Drop the fragment, then the query string.
    let input = input.split('#').next().unwrap_or_default();
    let input = input.split('?').next().unwrap_or_default();

    // Find the `room` path segment; everything before it (origin) is ignored.
    let segments: Vec<&str> = input.split('/').collect();
    let room_index = segments.iter().position(|s| *s == "room")?;
    let rest: Vec<&str> = segments[room_index + 1..]
        .iter()
        .copied()
        .filter(|s| !s.is_empty())
        .collect();

    let (room_type, room_name, password) = match rest.as_slice() {
        [room_name, password] => (None, *room_name, *password),
        [slug, room_name, password] => (RoomType::from_url_slug(slug), *room_name, *password),
        _ => return None,
    };

    let room_name = percent_decode(room_name)?;
    let hashed_password = percent_decode(password)?;
    if room_name.is_empty() || hashed_password.is_empty() {
        return None;
    }
    Some(RoomUrl {
        room_name,
        hashed_password,
        room_type,
    })
}

/// Decodes `%XX` escapes (as browsers add when copying a URL with spaces or non-ASCII
/// characters). Returns `None` on a malformed escape or invalid UTF-8.
fn percent_decode(segment: &str) -> Option<String> {
    let bytes = segment.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = segment.get(i + 1..i + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
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
    fn accepts_bare_path_whitespace_trailing_slash_query_and_fragment() {
        assert_eq!(
            parse_room_url("  /room/sizing/team-a/abc123/?x=1#top \n"),
            url(Some(RoomType::Sizing), "team-a", "abc123")
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
}
