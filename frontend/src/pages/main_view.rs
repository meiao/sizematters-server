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

use crate::components::room::Room;
use crate::stores::{RoomStore, UserStore};
use crate::ws::WsContext;
use leptos::prelude::*;

#[component]
pub fn MainPage() -> impl IntoView {
    let user_store = expect_context::<UserStore>();
    let room_store = expect_context::<RoomStore>();
    let ws_ctx = expect_context::<WsContext>();

    let is_default_name = Memo::new(move |_| {
        let own_id = user_store.own_user_id();
        user_store.user_signal().with(|users| {
            users.get(&own_id).map(|u| u.name == "Shirtless Muppet").unwrap_or(false)
        })
    });

    let banner_dismissed = RwSignal::new(false);
    let rooms = room_store.rooms_signal();
    let last_error = ws_ctx.last_error();

    let dismiss_banner = move |_| {
        banner_dismissed.set(true);
    };

    view! {
        <div class="main" on:click=dismiss_banner>
            // Error toast from server messages
            <Show when=move || last_error.get().is_some()>
                <div class="card error-toast" id="error-toast" role="alert">
                    <div class="card-content">
                        <span class="material-icons" aria-hidden="true">"error"</span>
                        <div>{move || last_error.get().unwrap_or_default()}</div>
                        <button
                            class="btn btn-icon"
                            aria-label="Dismiss error"
                            on:click=move |e| {
                                e.stop_propagation();
                                ws_ctx.clear_error();
                            }
                        >
                            <span class="material-icons" aria-hidden="true">"close"</span>
                        </button>
                    </div>
                </div>
            </Show>

            // Banner prompting new users to set their name
            <Show when=move || is_default_name.get() && !banner_dismissed.get()>
                <div class="card" id="change-name-alert" role="status">
                    <div class="card-content">
                        <span class="material-icons" aria-hidden="true">"arrow_back"</span>
                        <div>
                            "Hi " <b>"Shirtless Muppet"</b> "."<br />
                            "You can change your name by clicking it here to the left."
                        </div>
                    </div>
                </div>
            </Show>

            <div id="rooms">
                {move || rooms.get().iter().map(|room| {
                    let rn = room.room_name.clone();
                    view! { <Room room_name=rn /> }
                }).collect_view()}

                <Show when=move || rooms.get().is_empty()>
                    <div class="empty-state">
                        <span class="material-icons empty-icon">"new_releases"</span>
                        <h2>"Join a room"</h2>
                        <p>"To get the most of this website you should join a room, or create one. Do so by pressing the '+' over there."</p>
                    </div>
                </Show>
            </div>
        </div>
    }
}
