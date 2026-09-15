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

use crate::ws::WsContext;
use leptos::prelude::*;

/// Shown in the sidebar when not connected. Clicking "Enter" initiates the WebSocket connection.
///
/// The actual "navigate to /main once connected" and "error on timeout" logic
/// lives in `MenuRouter` (app.rs), which stays mounted across the swap from
/// `NoMenu` to `MainMenu` — this component gets unmounted the instant the
/// connection succeeds, so it must not be the thing reacting to that event.
#[component]
pub fn NoMenu() -> impl IntoView {
    let ws = expect_context::<WsContext>();
    let initiated = Memo::new(move |_| ws.is_initiated());

    let on_enter = move |_| {
        ws.connect();
    };

    view! {
        <Show when=move || initiated.get()>
            <p>"Connecting..."</p>
        </Show>
        <Show when=move || !initiated.get()>
            <button class="btn btn-raised btn-primary" on:click=on_enter>
                "Enter"
            </button>
        </Show>
    }
}
