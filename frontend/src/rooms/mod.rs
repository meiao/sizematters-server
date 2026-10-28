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

//! Everything specific to a room type lives in its own module (`sizing`, `jyanken`): the view,
//! its cards, its store and the handler for its server messages. Pieces used by several room
//! types live in `common`.
//!
//! This module is the only place that lists the room types. The rest of the app goes through
//! [`RoomView`] and [`RoomStores`] and doesn't need to know which games exist.

pub mod common;
pub mod jyanken;
pub mod sizing;

use crate::stores::{RoomStore, UserStore};
use jyanken::{JyankenRoom, JyankenStore};
use leptos::prelude::*;
use sizematters_shared::RoomType;
use sizing::{SizingRoom, VoteStore};

/// What a room message handler may touch besides its own store.
#[derive(Clone, Copy)]
pub struct HandlerCtx {
    pub room_store: RoomStore,
    pub user_store: UserStore,
    /// Error shown to the user, e.g. when voting after the round is over.
    pub last_error: RwSignal<Option<String>>,
}

/// The stores of every room type. Each one holds the state of all rooms of its type, keyed by
/// room name.
#[derive(Clone, Copy)]
pub struct RoomStores {
    sizing: VoteStore,
    jyanken: JyankenStore,
}

impl RoomStores {
    pub fn new() -> Self {
        Self {
            sizing: VoteStore::new(),
            jyanken: JyankenStore::new(),
        }
    }

    /// Provide this, and each room type's store for its views, as context.
    pub fn provide(self) {
        provide_context(self);
        provide_context(self.sizing);
        provide_context(self.jyanken);
    }

    /// Start tracking a room we joined. `user_ids` are the users already in it.
    pub fn room_joined(&self, room_type: RoomType, room_name: &str, user_ids: &[String]) {
        match room_type {
            RoomType::Sizing => self.sizing.room_joined(room_name, user_ids),
            RoomType::Jyanken => self.jyanken.room_joined(room_name, user_ids),
        }
    }

    pub fn user_joined(&self, room_type: RoomType, room_name: &str, user_id: &str) {
        match room_type {
            RoomType::Sizing => self.sizing.user_joined(room_name, user_id),
            RoomType::Jyanken => self.jyanken.user_joined(room_name, user_id),
        }
    }

    pub fn user_left(&self, room_type: RoomType, room_name: &str, user_id: &str) {
        match room_type {
            RoomType::Sizing => self.sizing.user_left(room_name, user_id),
            RoomType::Jyanken => self.jyanken.user_left(room_name, user_id),
        }
    }

    /// Handle the payload of a `RoomMessage` sent to a room of the given type.
    pub fn handle_message(
        &self,
        room_type: RoomType,
        room_name: &str,
        payload: &str,
        ctx: HandlerCtx,
    ) {
        match room_type {
            RoomType::Sizing => sizing::handle_message(payload, room_name, self.sizing, ctx),
            RoomType::Jyanken => jyanken::handle_message(payload, room_name, self.jyanken, ctx),
        }
    }

    /// Forget a room we left.
    pub fn leave(&self, room_name: &str) {
        self.sizing.leave_room(room_name);
        self.jyanken.leave_room(room_name);
    }

    /// Forget every room, e.g. before rejoining them after a reconnect.
    pub fn clear(&self) {
        self.sizing.clear();
        self.jyanken.clear();
    }
}

/// The view of a room, picked by its type.
#[component]
pub fn RoomView(room_name: String, room_type: RoomType) -> impl IntoView {
    match room_type {
        RoomType::Sizing => view! { <SizingRoom room_name=room_name /> }.into_any(),
        RoomType::Jyanken => view! { <JyankenRoom room_name=room_name /> }.into_any(),
    }
}

#[cfg(test)]
mod rooms_test;
