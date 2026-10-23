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

use super::card::VoteCard;
use super::VoteStore;
use crate::collate::compare_names;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{CardContent, CardFooter};
use crate::rooms::common::{send_room_msg, RoomShell, SpectatorList};
use crate::stores::{RoomStore, UserStore};
use icons::{Eye, RotateCcw, Shuffle};
use leptos::prelude::*;
use sizematters_shared::messages::sizing::{SizingRequestMessage, VotingSystem};
use sizematters_shared::UserData;
use strum::IntoEnumIterator;

#[component]
pub fn SizingRoom(room_name: String) -> impl IntoView {
    let room_store = expect_context::<RoomStore>();
    let vote_store = expect_context::<VoteStore>();
    let user_store = expect_context::<UserStore>();

    let rn = room_name.clone();
    let room_status = Memo::new(move |_| {
        room_store
            .rooms_signal()
            .with(|rooms| rooms.iter().find(|r| r.room_name == rn).cloned())
    });
    let users = move || room_status.get().map(|r| r.users).unwrap_or_default();
    let selected_user = move || room_status.get().and_then(|r| r.selected_user);

    let rn = room_name.clone();
    let results_revealed = Memo::new(move |_| vote_store.is_revealed(&rn));

    // Voter cards, sorted by name. Re-sorts when someone is renamed, since the room's
    // user list is updated on UserUpdated.
    let rn = room_name.clone();
    let voters = Memo::new(move |_| -> Vec<UserData> {
        let mut voters: Vec<UserData> = users()
            .into_iter()
            .filter(|u| !vote_store.is_spectator(&rn, &u.user_id))
            .collect();
        voters.sort_by(|a, b| compare_names(&a.name, &b.name));
        voters
    });

    let rn = room_name.clone();
    let spectators = Memo::new(move |_| -> Vec<UserData> {
        users()
            .into_iter()
            .filter(|u| vote_store.is_spectator(&rn, &u.user_id))
            .collect()
    });

    let rn = room_name.clone();
    let own_is_spectator = Memo::new(move |_| {
        let own_id = user_store.own_user_id();
        vote_store.is_spectator(&rn, &own_id)
    });

    let rn = room_name.clone();
    let own_vote_value = Memo::new(move |_| {
        let own_id = user_store.own_user_id();
        vote_store.votes_signal().with(|votes| {
            votes
                .get(&rn)
                .and_then(|rv| rv.get(&own_id))
                .and_then(|v| v.value.clone())
        })
    });

    let rn = room_name.clone();
    let voting_system = Memo::new(move |_| vote_store.voting_system(&rn));

    let rn = StoredValue::new(room_name.clone());
    let room_msg = move |msg: SizingRequestMessage| send_room_msg(&rn.read_value(), &msg);

    view! {
        <RoomShell room_name=room_name.clone()>
            <CardContent class="flex items-start gap-4 py-6">
                <SpectatorList spectators=spectators />
                <div class="flex flex-1 flex-wrap justify-center gap-4">
                    <For
                        each=move || voters.get()
                        key=|user| user.user_id.clone()
                        let:user
                    >
                        {
                            let uid = user.user_id.clone();
                            view! {
                                <VoteCard
                                    user_id=user.user_id
                                    room_name=room_name.clone()
                                    revealed=results_revealed
                                    selected=Signal::derive(move || selected_user() == Some(uid.clone()))
                                />
                            }
                        }
                    </For>
                </div>
            </CardContent>
            <CardFooter class="items-center gap-4 border-t pt-4" attr:role="group" attr:aria-label="Vote buttons">
                <div class="flex shrink-0 items-center gap-2 lg:flex-1">
                    <label for="voting-system-select" class="text-sm whitespace-nowrap">"Voting system"</label>
                    <select
                        id="voting-system-select"
                        class="text-foreground border-input flex h-9 min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-2 md:text-sm"
                        on:change=move |e| {
                            let index: usize = event_target_value(&e).parse().unwrap_or(0);
                            if let Some(new_system) = VotingSystem::iter().nth(index) {
                                room_msg(SizingRequestMessage::ChangeVotingSystem { new_system });
                            }
                        }
                    >
                        {VotingSystem::iter()
                            .enumerate()
                            .map(|(index, system)| {
                                view! {
                                    <option
                                        value=index.to_string()
                                        selected=move || voting_system.get() == system
                                    >
                                        {system.display_name()}
                                    </option>
                                }
                            })
                            .collect_view()}
                    </select>
                </div>
                <div class="flex min-w-0 flex-1 flex-wrap items-center justify-center gap-2 lg:flex-none">
                    {move || if results_revealed.get() {
                        view! {
                            <Button
                                variant=ButtonVariant::Success
                                on:click=move |_| room_msg(SizingRequestMessage::NewVote)
                            >
                                <RotateCcw />
                                "New vote"
                            </Button>
                            <Button on:click=move |_| room_msg(SizingRequestMessage::Randomize)>
                                <Shuffle />
                                "Randomize"
                            </Button>
                        }.into_any()
                    } else if own_is_spectator.get() {
                        view! {
                            <Button on:click=move |_| room_msg(SizingRequestMessage::RegisterToVote)>
                                <Eye />
                                "Register to vote"
                            </Button>
                        }.into_any()
                    } else {
                        // Keyed by value so the buttons (and their click handlers) are rebuilt
                        // when the voting system changes, rather than patched in place.
                        view! {
                            <For
                                each=move || voting_system.get().values().to_vec()
                                key=|value| *value
                                let:value
                            >
                                {
                                    let label = format!("Vote {}", value);
                                    let is_own_vote = move || own_vote_value.get().as_deref() == Some(value);
                                    view! {
                                        <Button
                                            variant=Signal::derive(move || {
                                                if is_own_vote() {
                                                    ButtonVariant::Success
                                                } else {
                                                    ButtonVariant::Outline
                                                }
                                            })
                                            class="h-10 min-w-10 rounded-full px-3 text-base font-semibold sm:h-14 sm:min-w-14 sm:text-xl"
                                            attr:aria-label=label
                                            attr:aria-pressed=move || is_own_vote().to_string()
                                            on:click=move |_| room_msg(SizingRequestMessage::Vote { size: value.to_string() })
                                        >
                                            {value}
                                        </Button>
                                    }
                                }
                            </For>
                        }.into_any()
                    }}
                </div>
                // On wide screens, balances the select on the left so the buttons stay centered.
                <div class="hidden lg:block lg:flex-1"></div>
            </CardFooter>
        </RoomShell>
    }
}
