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

use serde::{Deserialize, Serialize};
use strum::EnumIter;
use crate::avatar::AvatarStyle;

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
}
