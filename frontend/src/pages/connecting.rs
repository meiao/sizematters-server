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

use crate::ws::{self, WsContext};
use icons::LoaderCircle;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use sizematters_shared::RoomType;

/// Page shown when navigating to a shareable room URL:
/// - /room/:room_type/:room_name/:password creates the room if it does not exist yet,
///   or joins it if it does;
/// - /room/:room_name/:password (legacy, no room type) can only join an existing room.
///
/// Connects the WebSocket, creates/joins the room, then redirects to /main.
#[component]
pub fn ConnectingPage() -> impl IntoView {
    let ws_ctx = expect_context::<WsContext>();
    let params = use_params_map();
    let navigate = use_navigate();

    let room_name = params.get_untracked().get("room_name").unwrap_or_default();
    let password = params.get_untracked().get("password").unwrap_or_default();
    // An unknown room type falls back to joining, which still works for existing rooms.
    let room_type = params
        .get_untracked()
        .get("room_type")
        .and_then(|slug| RoomType::from_url_slug(&slug));

    // Start connection on mount (runs once since no tracked signals)
    Effect::new(move |ran: Option<bool>| {
        // Only run once
        if ran == Some(true) {
            return true;
        }

        if !ws_ctx.is_connected() {
            ws_ctx.connect();
        }
        true
    });

    // Reactively wait for connection, then join room and navigate
    let rn2 = room_name.clone();
    let pw2 = password.clone();
    let nav2 = navigate.clone();
    let joined = RwSignal::new(false);
    Effect::new(move |_| {
        if ws_ctx.is_connected() && !joined.get_untracked() {
            joined.set(true);
            ws::ws_register();
            if !rn2.is_empty() && !pw2.is_empty() {
                match room_type {
                    Some(room_type) => {
                        ws::ws_create_room(rn2.clone(), pw2.clone(), true, room_type)
                    }
                    None => ws::ws_join_room(rn2.clone(), pw2.clone(), true),
                }
            }
            nav2("/main", Default::default());
        }
    });

    // Timeout fallback for connection failure
    let nav3 = navigate.clone();
    set_timeout(
        move || {
            if !ws_ctx.is_connected() {
                nav3("/error/connection", Default::default());
            }
        },
        std::time::Duration::from_secs(5),
    );

    view! {
        <p class="flex items-center gap-2 text-muted-foreground" role="status">
            <LoaderCircle class="size-4 animate-spin" />
            "Connecting..."
        </p>
    }
}
