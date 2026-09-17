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
use leptos_router::hooks::use_params_map;

#[component]
pub fn ErrorPage() -> impl IntoView {
    let params = use_params_map();

    let error_type = Memo::new(move |_| params.get().get("error_type").unwrap_or_default());

    view! {
        <div class="error">
            <Show when=move || error_type.get() == "connection">
                <div>
                    "There was a problem connecting the websocket."<br />
                    "Go kick (with your fists) someone in your IT deparment and tell them it is a life or death situation."
                </div>
            </Show>
        </div>
    }
}
