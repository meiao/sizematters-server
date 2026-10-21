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

//! Jyanken rooms: rock-paper-scissors. Everyone shoots a weapon, the weapons are revealed once
//! everyone shot, and a scoreboard keeps the wins.

mod card;
mod messages;
mod scoreboard;
mod store;
mod view;
mod weapon;

pub use messages::handle_message;
pub use store::JyankenStore;
pub use view::JyankenRoom;
