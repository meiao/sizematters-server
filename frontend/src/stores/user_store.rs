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
use sizematters_shared::UserData;
use std::collections::HashMap;

/// Reactive store for user data (own identity + all known users).
#[derive(Clone, Copy)]
pub struct UserStore {
    users: RwSignal<HashMap<String, UserData>>,
    own_user_id: RwSignal<String>,
}

impl UserStore {
    pub fn new() -> Self {
        Self {
            users: RwSignal::new(HashMap::new()),
            own_user_id: RwSignal::new(String::new()),
        }
    }

    /// Called when the server sends OwnData with our identity.
    pub fn own_data(&self, user: UserData) {
        self.own_user_id.set(user.user_id.clone());
        self.users.update(|m| {
            m.insert(user.user_id.clone(), user);
        });
    }

    pub fn user_updated(&self, user: UserData) {
        self.users.update(|m| {
            m.insert(user.user_id.clone(), user);
        });
    }

    pub fn room_joined(&self, users: &[UserData]) {
        self.users.update(|m| {
            for user in users {
                m.insert(user.user_id.clone(), user.clone());
            }
        });
    }

    /// Non-reactive read for use outside reactive contexts (e.g. WebSocket handler).
    pub fn user_untracked(&self, user_id: &str) -> Option<UserData> {
        self.users.get_untracked().get(user_id).cloned()
    }

    pub fn user_signal(&self) -> RwSignal<HashMap<String, UserData>> {
        self.users
    }

    /// Tracked read — use inside Memos/Effects to re-run when own_user_id changes.
    pub fn own_user_id(&self) -> String {
        self.own_user_id.get()
    }

    /// Non-reactive read for use outside reactive contexts (e.g. WebSocket handler).
    pub fn own_user_id_untracked(&self) -> String {
        self.own_user_id.get_untracked()
    }
}
