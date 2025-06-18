/*
 * SizeMatters - a ticket sizing util
 * Copyright (C) 2025 Andre Onuki
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

use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::hash::Hash;

#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct UserData {
    pub user_id: String,
    pub name: String,
    pub gravatar_id: String,
}

impl UserData {
    pub fn new(user_id: String) -> UserData {
        let name = "Shirtless Muppet".to_string();
        let gravatar_id = format!("{:x}", md5::compute(user_id.clone()));
        UserData {
            user_id,
            name,
            gravatar_id,
        }
    }

    pub fn set_avatar(&mut self, avatar: &str) {
        self.gravatar_id = format!("{:x}", md5::compute(avatar));
    }
}
