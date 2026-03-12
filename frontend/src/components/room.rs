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

use crate::components::user_card::UserCard;
use crate::stores::{RoomStore, UserStore, VoteStore};
use crate::ws;
use leptos::prelude::*;

/// Fibonacci-like vote values used in planning poker.
const NUMBERS: &[u64] = &[0, 1, 2, 3, 5, 8, 13, 21];

#[component]
pub fn Room(room_name: String) -> impl IntoView {
    let room_store = expect_context::<RoomStore>();
    let vote_store = expect_context::<VoteStore>();
    let user_store = expect_context::<UserStore>();

    let rn = room_name.clone();
    let room_status = Memo::new(move |_| {
        let rooms = room_store.rooms_signal().get();
        rooms.iter().find(|r| r.room_name == rn).cloned()
    });

    let rn2 = room_name.clone();
    let voting_done = Memo::new(move |_| {
        vote_store.is_voting_done(&rn2)
    });

    let rn3 = room_name.clone();
    let users = move || {
        room_status.get().map(|r| r.users.clone()).unwrap_or_default()
    };

    let selected_user = move || {
        room_status.get().and_then(|r| r.selected_user.clone())
    };

    let rn5 = room_name.clone();
    let own_vote_value = Memo::new(move |_| {
        let own_id = user_store.own_user_id();
        let votes = vote_store.votes_signal().get();
        votes
            .get(&rn5)
            .and_then(|rv| rv.get(&own_id))
            .and_then(|v| v.value)
    });

    let room_name_display = room_name.clone();
    let rn_vote = room_name.clone();
    let rn_new = room_name.clone();
    let rn_rand = room_name.clone();

    let link_copied = RwSignal::new(false);

    let rn_link = room_name.clone();
    let room_link = Memo::new(move |_| {
        let hp = room_store.rooms_signal().get()
            .iter()
            .find(|r| r.room_name == rn_link)
            .map(|r| r.hashed_password.clone())
            .unwrap_or_default();
        let origin = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_default();
        format!("{}/room/{}/{}", origin, rn_link, hp)
    });

    view! {
        <div class="card room">
            <div class="card-header header">
                <div class="card-title">
                    {room_name_display} " - " {move || room_link.get()}
                    <button
                        class="btn btn-icon"
                        style="color: #fff;"
                        title="Copy room link"
                        aria-label="Copy room link"
                        on:click=move |_| {
                            let link = room_link.get();
                            if let Some(window) = web_sys::window() {
                                let clipboard = window.navigator().clipboard();
                                let _ = clipboard.write_text(&link);
                                link_copied.set(true);
                                set_timeout(move || link_copied.set(false), std::time::Duration::from_secs(2));
                            }
                        }
                    >
                        <span class="material-icons" aria-hidden="true">
                            {move || if link_copied.get() { "check" } else { "link" }}
                        </span>
                    </button>
                </div>
            </div>
            <div class="card-content user-space">
                <For
                    each=move || users()
                    key=|user| user.user_id.clone()
                    let:user
                >
                    {
                        let rn = rn3.clone();
                        let vd = voting_done;
                        let uid = user.user_id.clone();
                        let uid2 = user.user_id.clone();
                        view! {
                            <div
                                class="user"
                                class:show-vote=move || vd.get()
                                class:user-selected=move || selected_user() == Some(uid2.clone())
                            >
                                <UserCard user_id=uid.clone() room_name=rn.clone() />
                            </div>
                        }
                    }
                </For>
            </div>
            <div class="card-actions" role="group" aria-label="Vote buttons">
                {NUMBERS.iter().map(|&num| {
                    let rn = rn_vote.clone();
                    let vote_val = own_vote_value;
                    let label = format!("Vote {}", num);
                    view! {
                        <button
                            class="btn btn-icon vote-button"
                            class:btn-accent=move || vote_val.get() == Some(num)
                            class:btn-primary=move || vote_val.get() != Some(num)
                            aria-label=label
                            on:click=move |_| {
                                ws::ws_vote(rn.clone(), num);
                            }
                        >
                            {num}
                        </button>
                    }
                }).collect_view()}
                <Show when=move || voting_done.get()>
                    {
                        let rn = rn_new.clone();
                        view! {
                            <button
                                class="btn btn-accent btn-raised"
                                on:click=move |_| {
                                    ws::ws_new_vote(rn.clone());
                                }
                            >
                                "New vote"
                            </button>
                        }
                    }
                </Show>
                <Show when=move || voting_done.get()>
                    {
                        let rn = rn_rand.clone();
                        view! {
                            <button
                                class="btn btn-primary btn-raised"
                                on:click=move |_| {
                                    ws::ws_randomize(rn.clone());
                                }
                            >
                                "Randomize"
                            </button>
                        }
                    }
                </Show>
            </div>
        </div>
    }
}
