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
use crate::components::room_list::{leave_all_and_join, RoomList};
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
        users.get(&own_id).map(|u| u.name.clone()).unwrap_or("not connected".to_string())
    });

    let img_url = Memo::new(move |_| {
        let own_id = user_store.own_user_id();
        let users = user_store.user_signal().get();
        users.get(&own_id).map(|u| {
            format!("https://www.gravatar.com/avatar/{}?d=retro", u.gravatar_id)
        }).unwrap_or_default()
    });

    // Register on creation
    ws::ws_register();

    view! {
        <div>
            <PromptDialog
                title="What's your name?"
                show=show_name_dialog
                initial_value=Signal::derive(move || {
                    let n = name.get();
                    if n == "Shirtless Muppet" || n == "not connected" { String::new() } else { n }
                })
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
                on_confirm=Callback::new(move |(room_name, password): (String, String)| {
                    leave_all_and_join(room_store, vote_store, room_name, password, false);
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

            <RoomList show_room_dialog=show_room_dialog />
        </div>
    }
}
