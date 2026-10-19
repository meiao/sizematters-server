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

pub mod avatar;
pub mod messages;

use crate::avatar::AvatarStyle;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct UserData {
    pub user_id: String,
    pub name: String,
    pub avatar: String,
    pub avatar_style: AvatarStyle,
}

impl UserData {
    pub fn new(user_id: String, avatar: String) -> UserData {
        let name = "Shirtless Muppet".to_string();
        let avatar_style = AvatarStyle::default();
        UserData {
            user_id,
            name,
            avatar,
            avatar_style,
        }
    }

    pub fn set_avatar(&mut self, avatar: &str) {
        self.avatar = avatar.to_string();
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Hash, Eq, PartialEq, EnumIter)]
pub enum RoomType {
    Sizing,
    Jyanken,
}

impl RoomType {
    /// A human-readable label, for UIs that let the user pick a room type.
    pub fn display_name(&self) -> &'static str {
        match self {
            RoomType::Sizing => "Sizing",
            RoomType::Jyanken => "Jyanken (Rock Paper Scissor)",
        }
    }

    /// The identifier used for this room type in shareable room URLs.
    pub fn url_slug(&self) -> &'static str {
        match self {
            RoomType::Sizing => "sizing",
            RoomType::Jyanken => "jyanken",
        }
    }

    /// Parses a room type from its URL identifier (case-insensitive).
    pub fn from_url_slug(slug: &str) -> Option<RoomType> {
        RoomType::iter().find(|room_type| room_type.url_slug().eq_ignore_ascii_case(slug))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
