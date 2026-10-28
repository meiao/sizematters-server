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
use sizematters_shared::avatar::AvatarStyle;

pub(crate) fn default_avatar(user_id: &str) -> String {
    let style = Style::from_str(dicebear_styles::IDENTICON).unwrap();
    let avatar = Avatar::new(
        &style,
        json!({
            "seed": user_id
        }),
    )
    .unwrap();

    avatar.to_data_uri()
}

pub(crate) fn styled_avatar(user_id: &str, style: &AvatarStyle) -> String {
    let options = match style {
        AvatarStyle::Blobs
        | AvatarStyle::Glass
        | AvatarStyle::Pixelbot
        | AvatarStyle::Shapes
        | AvatarStyle::Squircles
        | AvatarStyle::Waves => json!({"seed": user_id, "animationVariant": ["fastest"]}),
        _ => json!({"seed": user_id}),
    };
    let style = get_style(style);
    Avatar::new(&style, options).unwrap().to_data_uri()
}

fn get_style(style: &AvatarStyle) -> Style {
    match style {
        AvatarStyle::Blobs => Style::from_str(dicebear_styles::BLOBS).unwrap(),
        AvatarStyle::Glass => Style::from_str(dicebear_styles::GLASS).unwrap(),
        AvatarStyle::Identicon => Style::from_str(dicebear_styles::IDENTICON).unwrap(),
        AvatarStyle::Squircles => Style::from_str(dicebear_styles::SQUIRCLES).unwrap(),
        AvatarStyle::Shapes => Style::from_str(dicebear_styles::SHAPES).unwrap(),
        AvatarStyle::Waves => Style::from_str(dicebear_styles::WAVES).unwrap(),
        AvatarStyle::Pixelbot => Style::from_str(dicebear_styles::PIXELBOT).unwrap(),
    }
}

#[cfg(test)]
mod avatar_test;
