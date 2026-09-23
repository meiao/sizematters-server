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

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
use crate::components::user_card::UserCard;
use crate::stores::{RoomStore, UserStore, VoteStore};
use crate::ws;
use icons::{Check, Link, RotateCcw, Shuffle};
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
    let voting_done = Memo::new(move |_| vote_store.is_voting_done(&rn2));

    let rn3 = room_name.clone();
    let users = move || {
        room_status
            .get()
            .map(|r| r.users.clone())
            .unwrap_or_default()
    };

    let selected_user = move || room_status.get().and_then(|r| r.selected_user.clone());

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
        let hp = room_store
            .rooms_signal()
            .get()
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
        <Card class="gap-0 overflow-hidden pt-0">
            <CardHeader class="grid grid-cols-[minmax(0,1fr)_auto] bg-black py-4 text-white dark:border-b dark:border-white/10">
                <CardTitle class="text-lg">{room_name_display}</CardTitle>
                <CardDescription class="w-full truncate text-neutral-400">
                    {move || room_link.get()}
                </CardDescription>
                <CardAction class="col-start-2 row-span-2 row-start-1 self-center">
                    <Button
                        variant=ButtonVariant::Ghost
                        size=ButtonSize::Icon
                        class="text-white hover:bg-white/15 hover:text-white"
                        attr:title="Copy room link"
                        attr:aria-label="Copy room link"
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
                        {move || if link_copied.get() {
                            view! { <Check /> }.into_any()
                        } else {
                            view! { <Link /> }.into_any()
                        }}
                    </Button>
                </CardAction>
            </CardHeader>
            <CardContent class="flex flex-wrap justify-center gap-4 py-6">
                <For
                    each=move || users()
                    key=|user| user.user_id.clone()
                    let:user
                >
                    {
                        let rn = rn3.clone();
                        let uid = user.user_id.clone();
                        let uid2 = user.user_id.clone();
                        view! {
                            <UserCard
                                user_id=uid
                                room_name=rn
                                revealed=voting_done
                                selected=Signal::derive(move || selected_user() == Some(uid2.clone()))
                            />
                        }
                    }
                </For>
            </CardContent>
            <CardFooter class="flex-wrap justify-center gap-2 border-t pt-4" attr:role="group" attr:aria-label="Vote buttons">
                {NUMBERS.iter().map(|&num| {
                    let rn = rn_vote.clone();
                    let vote_val = own_vote_value;
                    let label = format!("Vote {}", num);
                    view! {
                        <Button
                            variant=Signal::derive(move || {
                                if vote_val.get() == Some(num) {
                                    ButtonVariant::Success
                                } else {
                                    ButtonVariant::Outline
                                }
                            })
                            class="size-10 rounded-full text-base font-semibold sm:size-14 sm:text-xl"
                            attr:aria-label=label
                            attr:aria-pressed=move || (vote_val.get() == Some(num)).to_string()
                            on:click=move |_| {
                                ws::ws_vote(rn.clone(), num);
                            }
                        >
                            {num}
                        </Button>
                    }
                }).collect_view()}
                <Show when=move || voting_done.get()>
                    <div class="flex w-full justify-center gap-2 pt-2">
                        {
                            let rn = rn_new.clone();
                            view! {
                                <Button
                                    variant=ButtonVariant::Success
                                    on:click=move |_| {
                                        ws::ws_new_vote(rn.clone());
                                    }
                                >
                                    <RotateCcw />
                                    "New vote"
                                </Button>
                            }
                        }
                        {
                            let rn = rn_rand.clone();
                            view! {
                                <Button
                                    on:click=move |_| {
                                        ws::ws_randomize(rn.clone());
                                    }
                                >
                                    <Shuffle />
                                    "Randomize"
                                </Button>
                            }
                        }
                    </div>
                </Show>
            </CardFooter>
        </Card>
    }
}
