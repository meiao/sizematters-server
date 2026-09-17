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

use crate::components::dialogs::{PromptDialog, RoomDialog};
use crate::stores::{RoomStore, UserStore, VoteStore};
use crate::ws;
use leptos::prelude::*;

#[component]
pub fn MainMenu() -> impl IntoView {
    let user_store = expect_context::<UserStore>();
    let room_store = expect_context::<RoomStore>();
    let vote_store = expect_context::<VoteStore>();

    let show_name_dialog = RwSignal::new(false);
    let show_email_dialog = RwSignal::new(false);
    let show_room_dialog = RwSignal::new(false);

    let name = Memo::new(move |_| {
        let own_id = user_store.own_user_id();
        let users = user_store.user_signal().get();
        users
            .get(&own_id)
            .map(|u| u.name.clone())
            .unwrap_or("not connected".to_string())
    });

    let img_url = Memo::new(move |_| {
        let own_id = user_store.own_user_id();
        let users = user_store.user_signal().get();
        users
            .get(&own_id)
            .map(|u| format!("https://www.gravatar.com/avatar/{}?d=retro", u.gravatar_id))
            .unwrap_or_default()
    });

    let rooms = Memo::new(move |_| room_store.rooms_signal().get());

    // Register on creation
    ws::ws_register();

    view! {
        <div>
            <PromptDialog
                title="What's your name?"
                show=show_name_dialog
                on_confirm=Callback::new(move |name: String| {
                    ws::ws_set_name(name);
                })
            />
            <PromptDialog
                title="Profile picture"
                content=&[
                    "I tried my best to create a profile picture for you. Since you didn't like it, you can use your Gravatar image.",
                    "I will need your email for such. I promise I won't save/sell your email.",
                ]
                show=show_email_dialog
                on_confirm=Callback::new(|email: String| {
                    ws::ws_set_avatar(email);
                })
            />
            <RoomDialog
                show=show_room_dialog
                on_confirm=Callback::new(|(room_name, password): (String, String)| {
                    ws::ws_join_room(room_name, password, false);
                })
            />

            <div class="card">
                <div class="card-content" id="user-tag">
                    <div
                        id="user-avatar"
                        class="avatar"
                        role="button"
                        tabindex="0"
                        aria-label="Change profile picture"
                        on:click=move |_| show_email_dialog.set(true)
                        on:keydown=move |e: web_sys::KeyboardEvent| {
                            if e.key() == "Enter" || e.key() == " " {
                                e.prevent_default();
                                show_email_dialog.set(true);
                            }
                        }
                    >
                        {move || {
                            let url = img_url.get();
                            if url.is_empty() {
                                let n = name.get();
                                let first = n.chars().next().unwrap_or(' ');
                                view! { <span>{first.to_string()}</span> }.into_any()
                            } else {
                                view! { <img src=url alt="" /> }.into_any()
                            }
                        }}
                    </div>
                    <div
                        id="user-name"
                        role="button"
                        tabindex="0"
                        aria-label="Change your name"
                        on:click=move |_| show_name_dialog.set(true)
                        on:keydown=move |e: web_sys::KeyboardEvent| {
                            if e.key() == "Enter" || e.key() == " " {
                                e.prevent_default();
                                show_name_dialog.set(true);
                            }
                        }
                    >
                        {name}
                    </div>
                </div>
            </div>

            <div id="menu-rooms">
                <header>
                    "Rooms"
                    <button class="btn btn-icon btn-primary" aria-label="Create or join a room" on:click=move |_| show_room_dialog.set(true)>
                        <span class="material-icons" aria-hidden="true">"add"</span>
                    </button>
                </header>
                <Show when=move || rooms.get().is_empty()>
                    <div class="card">
                        <div class="card-content">
                            "Yes, the \"+\" over here"
                        </div>
                    </div>
                </Show>
                <div id="room-list">
                    <For
                        each=move || rooms.get()
                        key=|room| room.room_name.clone()
                        let:room
                    >
                        {
                            let rn = room.room_name.clone();
                            let rn2 = room.room_name.clone();
                            view! {
                                <div class="card">
                                    <div class="card-header">
                                        <div class="card-title">{rn.clone()}</div>
                                    </div>
                                    <div class="card-content">
                                        "Voting: " {room.votes_cast} "/" {room.users.len()}
                                    </div>
                                    <div class="card-actions">
                                        <button class="btn" on:click=move |_| {
                                            ws::ws_leave_room(rn2.clone());
                                            room_store.leave_room(&rn2);
                                            vote_store.leave_room(&rn2);
                                        }>"Leave"</button>
                                    </div>
                                </div>
                            }
                        }
                    </For>
                </div>
            </div>
        </div>
    }
}
