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

use icons::LoaderCircle;
use leptos::prelude::*;

/// Shown in the sidebar while the WebSocket connection is being established.
///
/// The connection itself is started, and the "navigate to /main once connected" and "error on
/// timeout" logic lives, in `MenuRouter` (app.rs), which stays mounted across the swap from
/// `NoMenu` to `MainMenu`.
#[component]
pub fn NoMenu() -> impl IntoView {
    view! {
        <p class="flex items-center justify-center gap-2 text-sm text-muted-foreground" role="status">
            <LoaderCircle class="size-4 animate-spin" />
            "Connecting..."
        </p>
    }
}
