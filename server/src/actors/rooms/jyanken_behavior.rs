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

use crate::actors::rooms::room::{RoomBehavior, RoomState};
use sizematters_shared::messages::jyanken::{
    JyankenRequestMessage, JyankenResponseMessage, JyankenWeapon,
};
use sizematters_shared::messages::ClientResponseMessage;
use std::collections::HashMap;
use std::sync::Arc;

pub(super) struct JyankenBehavior {
    weapon_map: HashMap<Arc<String>, JyankenWeapon>,
    game_over: bool,
    score: HashMap<Arc<String>, (u32, u32)>,
}

impl JyankenBehavior {
    pub(super) fn new() -> Self {
        JyankenBehavior {
            weapon_map: HashMap::new(),
            game_over: false,
            score: HashMap::new(),
        }
    }
}

impl RoomBehavior<JyankenRequestMessage> for JyankenBehavior {
    fn on_join_room(&mut self, user_id: &Arc<String>, room: &mut RoomState) {
        room.notify_user(user_id, self.game_status(room));
        if self.game_over {
            room.notify_user(user_id, self.game_results(room))
        }
    }

    fn on_leave_room(&mut self, user_id: &Arc<String>, room: &mut RoomState) {
        self.weapon_map.remove(user_id);
        self.score.remove(user_id);
        self.send_game_info(room);
    }

    fn on_specific_message(
        &mut self,
        user_id: Arc<String>,
        msg: JyankenRequestMessage,
        room: &mut RoomState,
    ) {
        match msg {
            JyankenRequestMessage::Shoot { weapon } => self.shoot(user_id, weapon, room),
            JyankenRequestMessage::NewGame => self.new_game(&user_id, room),
            JyankenRequestMessage::JoinGame => self.join_game(user_id, room),
            JyankenRequestMessage::RemoveFromGame { target_id } => {
                self.remove_from_game(user_id, target_id, room)
            }
        }
    }
}

impl JyankenBehavior {
    fn shoot(&mut self, user_id: Arc<String>, weapon: JyankenWeapon, room: &mut RoomState) {
        if !room.contains_user(&user_id) {
            println!("JyankenBehavior: User tried to play in a room he is not in.");
            return;
        }

        if self.game_over {
            room.notify_user(
                &user_id,
                Self::room_msg(room.room_name(), JyankenResponseMessage::GameOver),
            );
            return;
        }

        if room.is_spectator(&user_id) {
            room.notify_user(
                &user_id,
                ClientResponseMessage::Error {
                    msg: "You must join the game before shooting.".to_string(),
                },
            );
            return;
        }

        let msg = Self::room_msg(
            room.room_name(),
            JyankenResponseMessage::OwnShoot {
                weapon: weapon.clone(),
            },
        );
        room.notify_user(&user_id, msg);

        let already_shot = self.weapon_map.insert(user_id, weapon).is_some();
        if !already_shot {
            self.send_game_info(room);
        }
    }

    fn new_game(&mut self, user_id: &Arc<String>, room: &RoomState) {
        if !room.contains_user(user_id) {
            println!("JyankenBehavior: User tried to request new vote in a room they is not in.");
            return;
        }

        self.weapon_map.clear();
        self.game_over = false;

        let room_name = room.room_name();
        let msg = Self::room_msg(room_name.clone(), JyankenResponseMessage::NewGame);
        room.notify_users(msg);
    }

    fn join_game(&mut self, user_id: Arc<String>, room: &mut RoomState) {
        if !room.contains_user(&user_id) {
            return;
        }
        room.remove_spectator(user_id);
        self.send_game_info(room)
    }

    fn remove_from_game(&mut self, user_id: Arc<String>, target_id: String, room: &mut RoomState) {
        let target_id = Arc::new(target_id);
        if !room.contains_user(&target_id) {
            room.notify_user(
                &user_id,
                ClientResponseMessage::Error {
                    msg: "User not found.".to_string(),
                },
            );
            return;
        }
        if room.is_spectator(&target_id) {
            return;
        }
        self.weapon_map.remove(&target_id);
        room.add_spectator(target_id);
        self.send_game_info(room);
    }

    fn send_game_info(&mut self, room: &mut RoomState) {
        room.notify_users(self.game_status(room));
        if !self.game_over && self.all_shot(room) {
            self.game_over = true;
            self.update_score();
        }
        if self.game_over {
            room.notify_users(self.game_results(room));
        }
    }

    fn game_status(&self, room: &RoomState) -> ClientResponseMessage {
        let selected: HashMap<String, bool> = room
            .user_ids()
            .filter(|user_id| !room.is_spectator(user_id))
            .map(|user_id| ((**user_id).clone(), self.weapon_map.contains_key(user_id)))
            .collect();
        let spectators = room.spectators();
        Self::room_msg(
            room.room_name(),
            JyankenResponseMessage::GameStatus {
                selected,
                spectators,
            },
        )
    }

    fn game_results(&self, room: &RoomState) -> ClientResponseMessage {
        let weapons = self
            .weapon_map
            .iter()
            .map(|(user, weapon)| ((**user).clone(), weapon.clone()))
            .collect();
        let scoreboard = self
            .score
            .iter()
            .map(|(user, score)| ((**user).clone(), *score))
            .collect();
        Self::room_msg(
            room.room_name(),
            JyankenResponseMessage::GameResults {
                weapons,
                scoreboard,
            },
        )
    }

    fn update_score(&mut self) {
        let mut weapon_count: HashMap<JyankenWeapon, u32> = HashMap::new();
        for weapon in self.weapon_map.values() {
            *weapon_count.entry(weapon.clone()).or_insert(0) += 1
        }

        for (user, weapon) in &self.weapon_map {
            let wins = Self::calculate_wins(weapon, &weapon_count);
            match self.score.get(user) {
                None => self.score.insert(user.clone(), (wins, wins)),
                Some((_, total_wins)) => self.score.insert(user.clone(), (wins, wins + total_wins)),
            };
        }
    }

    fn calculate_wins(weapon: &JyankenWeapon, weapon_count: &HashMap<JyankenWeapon, u32>) -> u32 {
        match weapon {
            JyankenWeapon::Rock => Self::count(&JyankenWeapon::Scissor, weapon_count),
            JyankenWeapon::Paper => Self::count(&JyankenWeapon::Rock, weapon_count),
            JyankenWeapon::Scissor => Self::count(&JyankenWeapon::Paper, weapon_count),
        }
    }

    fn count(weapon: &JyankenWeapon, weapon_count: &HashMap<JyankenWeapon, u32>) -> u32 {
        weapon_count.get(weapon).copied().unwrap_or(0)
    }

    fn all_shot(&self, room: &RoomState) -> bool {
        let active_user_count = room.user_count() - room.spectator_count();
        active_user_count > 0 && self.weapon_map.len() == active_user_count
    }

    fn room_msg(room_name: String, msg: JyankenResponseMessage) -> ClientResponseMessage {
        if let Ok(payload) = serde_json::to_string(&msg) {
            ClientResponseMessage::RoomMessage { room_name, payload }
        } else {
            ClientResponseMessage::Error {
                msg: "This is awkward. I was not able to serialize a message.".to_string(),
            }
        }
    }
}

#[cfg(test)]
mod jyanken_behavior_test;
