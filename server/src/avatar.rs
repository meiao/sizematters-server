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

use dicebear_core::{Avatar, Style};
use serde_json::json;

pub(crate) fn default_avatar(user_id: &str) -> String {
    let style = Style::from_str(dicebear_styles::IDENTICON).unwrap();
    let avatar = Avatar::new(&style, json!({
        "seed": user_id
    })).unwrap();

    avatar.to_data_uri()
}