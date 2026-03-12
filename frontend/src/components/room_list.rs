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

use crate::storage;
use crate::storage::RecentRoom;
use crate::stores::{RoomStore, VoteStore};
use crate::ws;
use leptos::prelude::*;

pub fn leave_all_and_join(room_store: RoomStore, vote_store: VoteStore, room_name: String, password: String, password_is_hash: bool) {
    let active = room_store.rooms_signal().get_untracked();
    for r in &active {
        ws::ws_leave_room(r.room_name.clone());
        room_store.leave_room(&r.room_name);
        vote_store.leave_room(&r.room_name);
    }
    ws::ws_join_room(room_name, password, password_is_hash);
}

#[component]
pub fn RoomList(
    show_room_dialog: RwSignal<bool>,
) -> impl IntoView {
    let room_store = expect_context::<RoomStore>();

    let rooms = Memo::new(move |_| room_store.rooms_signal().get());

    let recent_rooms = RwSignal::new(storage::load_recent_rooms());

    // Refresh recent rooms whenever active rooms change
    Effect::new(move |_| {
        let _ = room_store.rooms_signal().get();
        recent_rooms.set(storage::load_recent_rooms());
    });

    // Recent rooms that aren't currently joined
    let filtered_recent = Memo::new(move |_| {
        let active: Vec<String> = rooms.get().iter().map(|r| r.room_name.clone()).collect();
        recent_rooms.get().into_iter().filter(|r| !active.contains(&r.room_name)).collect::<Vec<RecentRoom>>()
    });

    view! {
        <div id="menu-rooms">
            <header>
                "Rooms"
                <button class="btn btn-icon btn-primary" aria-label="Create or join a room" on:click=move |_| show_room_dialog.set(true)>
                    <span class="material-icons" aria-hidden="true">"add"</span>
                </button>
            </header>
            <Show when=move || rooms.get().is_empty() && filtered_recent.get().is_empty()>
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
                    <ActiveRoomItem room_name=room.room_name.clone() />
                </For>
            </div>

            <Show when=move || !filtered_recent.get().is_empty()>
                <div id="recent-room-list">
                    <For
                        each=move || filtered_recent.get()
                        key=|room| room.room_name.clone()
                        let:room
                    >
                        <RecentRoomItem
                            room_name=room.room_name.clone()
                            hashed_password=room.hashed_password.clone()
                            recent_rooms=recent_rooms
                        />
                    </For>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn ActiveRoomItem(
    room_name: String,
) -> impl IntoView {
    let room_store = expect_context::<RoomStore>();
    let vote_store = expect_context::<VoteStore>();
    let rn = room_name.clone();
    let rn2 = room_name.clone();

    let room_status = Memo::new(move |_| {
        room_store.rooms_signal().get()
            .into_iter()
            .find(|r| r.room_name == rn)
    });

    view! {
        <div class="card">
            <div class="card-header">
                <div class="card-title">{room_name}</div>
            </div>
            <div class="card-content">
                "Voting: "
                {move || room_status.get().map(|r| r.votes_cast).unwrap_or(0)}
                "/"
                {move || room_status.get().map(|r| r.users.len()).unwrap_or(0)}
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

#[component]
fn RecentRoomItem(
    room_name: String,
    hashed_password: String,
    recent_rooms: RwSignal<Vec<RecentRoom>>,
) -> impl IntoView {
    let room_store = expect_context::<RoomStore>();
    let vote_store = expect_context::<VoteStore>();
    let rn = room_name.clone();
    let hp = hashed_password.clone();
    let rn2 = room_name.clone();
    let hp2 = hashed_password.clone();
    let rn_delete = room_name.clone();

    let room_name_label = format!("Rejoin room {}", room_name);
    view! {
        <div
            class="card recent-room"
            role="button"
            tabindex="0"
            aria-label=room_name_label
            on:click=move |_| {
                leave_all_and_join(room_store, vote_store, rn.clone(), hp.clone(), true);
            }
            on:keydown=move |e: web_sys::KeyboardEvent| {
                if e.key() == "Enter" || e.key() == " " {
                    e.prevent_default();
                    leave_all_and_join(room_store, vote_store, rn2.clone(), hp2.clone(), true);
                }
            }
        >
            <div class="card-content">
                <span>{room_name}</span>
                <button
                    class="btn btn-icon recent-room-delete"
                    aria-label="Remove from recent rooms"
                    on:click=move |e| {
                        e.stop_propagation();
                        storage::remove_recent_room(&rn_delete);
                        recent_rooms.set(storage::load_recent_rooms());
                    }
                >
                    <span class="material-icons" aria-hidden="true">"delete"</span>
                </button>
            </div>
        </div>
    }
}
