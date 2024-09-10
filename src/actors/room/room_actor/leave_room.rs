/*
 * SizeMatters - a ticket sizing util
 * Copyright (C) 2020 Andre Onuki
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

use crate::actors::messages::{ClientResponseMessage, RoomMessage};
use crate::actors::room::RoomActor;
use actix::{ActorContext, Context};

impl RoomActor {
    pub(super) fn leave_room(&mut self, user_id: String, ctx: &mut Context<Self>) {
        let msg = ClientResponseMessage::UserLeft {
            user_id: user_id.clone(),
            room_name: self.name.clone(),
        };
        self.notify_users(msg);

        if self.active_user_map.contains_key(&user_id)
        {
            self.active_user_map.remove(&user_id);
        }
        else if self.passive_user_map.contains_key(&user_id)
        {
            self.passive_user_map.remove(&user_id);
        }

        self.vote_map.remove(&user_id);

        self.send_vote_info();

        if self.active_user_map.is_empty() {
            let msg = RoomMessage::RoomClosing {
                room_name: self.name.clone(),
            };
            self.notify_manager(msg);
            ctx.stop();
        }
    }
}
