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

use super::card::JyankenCard;
use super::scoreboard::Scoreboard;
use super::weapon::{weapon_label, WeaponIcon, WEAPONS};
use super::JyankenStore;
use crate::collate::compare_names;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::{CardContent, CardFooter};
use crate::rooms::common::{send_room_msg, RoomShell, SpectatorList};
use crate::stores::{RoomStore, UserStore};
use icons::{Eye, RotateCcw};
use leptos::prelude::*;
use sizematters_shared::messages::jyanken::JyankenRequestMessage;
use sizematters_shared::UserData;

#[component]
pub fn JyankenRoom(room_name: String) -> impl IntoView {
    let room_store = expect_context::<RoomStore>();
    let jyanken_store = expect_context::<JyankenStore>();
    let user_store = expect_context::<UserStore>();

    let rn = room_name.clone();
    let users = Memo::new(move |_| -> Vec<UserData> {
        room_store.rooms_signal().with(|rooms| {
            rooms
                .iter()
                .find(|r| r.room_name == rn)
                .map(|r| r.users.clone())
                .unwrap_or_default()
        })
    });

    let rn = room_name.clone();
    let state = Memo::new(move |_| jyanken_store.state(&rn));
    let revealed = Memo::new(move |_| state.with(|s| s.results.is_some()));

    let players = Memo::new(move |_| -> Vec<UserData> {
        let mut players: Vec<UserData> = state.with(|s| {
            users
                .get()
                .into_iter()
                .filter(|u| !s.spectators.contains(&u.user_id))
                .collect()
        });
        players.sort_by(|a, b| compare_names(&a.name, &b.name));
        players
    });
    let spectators = Memo::new(move |_| -> Vec<UserData> {
        state.with(|s| {
            users
                .get()
                .into_iter()
                .filter(|u| s.spectators.contains(&u.user_id))
                .collect()
        })
    });
    let own_is_spectator = Memo::new(move |_| {
        let own_id = user_store.own_user_id();
        state.with(|s| s.spectators.contains(&own_id))
    });

    let rn = StoredValue::new(room_name.clone());
    let room_msg = move |msg: JyankenRequestMessage| send_room_msg(&rn.read_value(), &msg);

    view! {
        <RoomShell room_name=room_name.clone()>
            <CardContent class="flex flex-col gap-6 py-6">
                <div class="flex items-start gap-4">
                    <SpectatorList spectators=spectators />
                    <div class="flex flex-1 flex-wrap justify-center gap-4">
                        <For
                            each=move || players.get()
                            key=|user| user.user_id.clone()
                            let:user
                        >
                            <JyankenCard
                                user_id=user.user_id
                                room_name=room_name.clone()
                                users=users
                                state=state
                            />
                        </For>
                    </div>
                    <Scoreboard state=state />
                </div>
            </CardContent>
            <CardFooter class="flex-wrap justify-center gap-2 border-t pt-4" attr:role="group" attr:aria-label="Weapons">
                {move || if own_is_spectator.get() {
                    view! {
                        <Button on:click=move |_| room_msg(JyankenRequestMessage::JoinGame)>
                            <Eye />
                            "Join the game"
                        </Button>
                    }.into_any()
                } else {
                    WEAPONS.iter().cloned().map(|weapon| {
                        let picked = weapon.clone();
                        let is_own_weapon =
                            Memo::new(move |_| state.with(|s| s.own_weapon.as_ref() == Some(&picked)));
                        let shoot = weapon.clone();
                        let label = format!("Shoot {}", weapon_label(&weapon));
                        view! {
                            <Button
                                variant=Signal::derive(move || {
                                    if is_own_weapon.get() {
                                        ButtonVariant::Success
                                    } else {
                                        ButtonVariant::Outline
                                    }
                                })
                                class="size-14 rounded-full sm:size-16"
                                attr:disabled=move || revealed.get()
                                attr:aria-label=label
                                attr:aria-pressed=move || is_own_weapon.get().to_string()
                                on:click=move |_| room_msg(JyankenRequestMessage::Shoot { weapon: shoot.clone() })
                            >
                                <WeaponIcon weapon=weapon class="text-2xl sm:text-3xl" />
                            </Button>
                        }
                    }).collect_view().into_any()
                }}
                <Show when=move || revealed.get()>
                    <div class="flex w-full justify-center pt-2">
                        <Button
                            variant=ButtonVariant::Success
                            on:click=move |_| room_msg(JyankenRequestMessage::NewGame)
                        >
                            <RotateCcw />
                            "New game"
                        </Button>
                    </div>
                </Show>
            </CardFooter>
        </RoomShell>
    }
}
