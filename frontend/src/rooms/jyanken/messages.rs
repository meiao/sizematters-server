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

use super::JyankenStore;
use crate::rooms::HandlerCtx;
use leptos::prelude::*;
use sizematters_shared::messages::jyanken::JyankenResponseMessage;

/// Handle the payload of a `RoomMessage` sent to a jyanken room.
pub fn handle_message(
    payload: &str,
    room_name: &str,
    jyanken_store: JyankenStore,
    ctx: HandlerCtx,
) {
    let msg = match serde_json::from_str::<JyankenResponseMessage>(payload) {
        Ok(msg) => msg,
        Err(err) => {
            log::warn!("Failed to parse jyanken room message: {}", err);
            return;
        }
    };
    match msg {
        JyankenResponseMessage::OwnShoot { weapon } => jyanken_store.own_shoot(room_name, weapon),
        JyankenResponseMessage::GameStatus {
            selected,
            spectators,
        } => jyanken_store.game_status(room_name, selected, spectators),
        JyankenResponseMessage::GameResults {
            weapons,
            scoreboard,
        } => jyanken_store.game_results(room_name, weapons, scoreboard),
        JyankenResponseMessage::NewGame => {
            ctx.room_store.clear_selected_user(room_name);
            jyanken_store.new_game(room_name);
        }
        JyankenResponseMessage::GameOver => {
            ctx.last_error
                .set(Some("The game is already over.".to_string()));
        }
    }
}
