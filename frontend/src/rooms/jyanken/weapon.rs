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

use leptos::prelude::*;
use sizematters_shared::messages::jyanken::JyankenWeapon;

pub const WEAPONS: [JyankenWeapon; 3] = [
    JyankenWeapon::Rock,
    JyankenWeapon::Paper,
    JyankenWeapon::Scissor,
];

pub fn weapon_label(weapon: &JyankenWeapon) -> &'static str {
    match weapon {
        JyankenWeapon::Rock => "Rock",
        JyankenWeapon::Paper => "Paper",
        JyankenWeapon::Scissor => "Scissors",
    }
}

#[component]
pub fn WeaponIcon(weapon: JyankenWeapon, #[prop(optional, into)] class: String) -> impl IntoView {
    let emoji = match weapon {
        JyankenWeapon::Rock => "✊",
        JyankenWeapon::Paper => "🖐️",
        JyankenWeapon::Scissor => "✌️",
    };
    view! {
        <span class=format!("leading-none {class}") aria-hidden="true">
            {emoji}
        </span>
    }
}

#[cfg(test)]
mod weapon_test;
