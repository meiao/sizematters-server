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

#[test]
fn weapons_are_listed_once_each() {
    let unique: HashSet<&JyankenWeapon> = WEAPONS.iter().collect();
    assert_eq!(unique.len(), WEAPONS.len());
}

#[test]
fn labels() {
    assert_eq!(weapon_label(&JyankenWeapon::Rock), "Rock");
    assert_eq!(weapon_label(&JyankenWeapon::Paper), "Paper");
    assert_eq!(weapon_label(&JyankenWeapon::Scissor), "Scissors");
}
