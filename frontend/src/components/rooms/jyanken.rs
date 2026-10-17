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

use crate::components::avatar::Avatar;
use crate::components::ui::badge::{Badge, BadgeSize, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use crate::stores::{JyankenStore, RoomStore, UserStore};
use crate::ws;
use icons::{Check, CircleHelp, Eye, EyeOff, Link, RotateCcw, Trophy};
use leptos::prelude::*;
use sizematters_shared::messages::jyanken::{JyankenRequestMessage, JyankenWeapon};
use sizematters_shared::UserData;

const WEAPONS: [JyankenWeapon; 3] = [
    JyankenWeapon::Rock,
    JyankenWeapon::Paper,
    JyankenWeapon::Scissor,
];

fn weapon_label(weapon: &JyankenWeapon) -> &'static str {
    match weapon {
        JyankenWeapon::Rock => "Rock",
        JyankenWeapon::Paper => "Paper",
        JyankenWeapon::Scissor => "Scissors",
    }
}

#[component]
fn WeaponIcon(weapon: JyankenWeapon, #[prop(optional, into)] class: String) -> impl IntoView {
    let emoji = match weapon {
        JyankenWeapon::Rock => "✊",
        JyankenWeapon::Paper => "🖐️",
        JyankenWeapon::Scissor => "✌️",
    };
    view! {
        <span class=format!("leading-none {class}") aria-hidden="true">
            {emoji}
        </span>
    }
}

#[component]
pub fn JyankenRoom(room_name: String) -> impl IntoView {
    let room_store = expect_context::<RoomStore>();
    let jyanken_store = expect_context::<JyankenStore>();
    let user_store = expect_context::<UserStore>();

    let rn = room_name.clone();
    let users = Memo::new(move |_| -> Vec<UserData> {
        room_store
            .rooms_signal()
            .get()
            .iter()
            .find(|r| r.room_name == rn)
            .map(|r| r.users.clone())
            .unwrap_or_default()
    });

    let rn_state = room_name.clone();
    let state = Memo::new(move |_| jyanken_store.state(&rn_state));
    let revealed = Memo::new(move |_| state.get().results.is_some());

    let players = Memo::new(move |_| -> Vec<UserData> {
        let s = state.get();
        let mut players: Vec<UserData> = users
            .get()
            .into_iter()
            .filter(|u| !s.spectators.contains(&u.user_id))
            .collect();
        players.sort_by_key(|u| u.name.to_lowercase());
        players
    });
    let spectators = Memo::new(move |_| -> Vec<UserData> {
        let s = state.get();
        users
            .get()
            .into_iter()
            .filter(|u| s.spectators.contains(&u.user_id))
            .collect()
    });
    let own_is_spectator = Memo::new(move |_| {
        let own_id = user_store.own_user_id();
        state.get().spectators.contains(&own_id)
    });

    // Scoreboard rows, best total first. Names come from the user store so
    // players who already left still show up under their last known name.
    let scoreboard = Memo::new(move |_| -> Vec<(String, u32, u32)> {
        let s = state.get();
        let known = user_store.user_signal().get();
        let mut rows: Vec<(String, u32, u32)> = s
            .scoreboard
            .iter()
            .map(|(id, (last, total))| {
                let name = known
                    .get(id)
                    .map(|u| u.name.clone())
                    .unwrap_or_else(|| id.clone());
                (name, *last, *total)
            })
            .collect();
        rows.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));
        rows
    });

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

    let rn_msg = StoredValue::new(room_name.clone());
    let room_msg = move |msg: JyankenRequestMessage| match serde_json::to_string(&msg) {
        Ok(payload) => ws::ws_room_msg(rn_msg.get_value(), payload),
        Err(err) => log::warn!("Failed to serialize message: {}", err),
    };

    let room_name_display = room_name.clone();

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
            <CardContent class="flex flex-col gap-6 py-6">
                <div class="flex items-start gap-4">
                    <Show when=move || !spectators.get().is_empty()>
                        <div class="flex w-36 shrink-0 flex-col gap-2 rounded-lg border p-3">
                            <h3 class="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                                <Eye class="size-3.5" />
                                "Spectators"
                            </h3>
                            <div class="flex flex-col gap-2">
                                <For
                                    each=move || spectators.get()
                                    key=|user| user.user_id.clone()
                                    let:user
                                >
                                    {
                                        let uid = user.user_id.clone();
                                        let current = Memo::new(move |_| {
                                            users.get().into_iter().find(|u| u.user_id == uid)
                                        });
                                        view! {
                                            <div class="flex items-center gap-2">
                                                <div class="size-6 shrink-0 overflow-hidden rounded-full">
                                                    {move || view! { <Avatar avatar=current.get().map(|u| u.avatar).unwrap_or_default() /> }}
                                                </div>
                                                <span class="truncate text-sm">{move || current.get().map(|u| u.name).unwrap_or_default()}</span>
                                            </div>
                                        }
                                    }
                                </For>
                            </div>
                        </div>
                    </Show>
                    <div class="flex flex-1 flex-wrap justify-center gap-4">
                        <For
                            each=move || players.get()
                            key=|user| user.user_id.clone()
                            let:user
                        >
                            {
                                let uid_remove = user.user_id.clone();
                                let uid_shot = user.user_id.clone();
                                let uid_weapon = user.user_id.clone();
                                let uid_score = user.user_id.clone();
                                let uid_user = user.user_id.clone();
                                let current = Memo::new(move |_| {
                                    users.get().into_iter().find(|u| u.user_id == uid_user)
                                });
                                let avatar = move || current.get().map(|u| u.avatar).unwrap_or_default();
                                let name = move || current.get().map(|u| u.name).unwrap_or_default();
                                let has_shot = move || {
                                    state.get().selected.get(&uid_shot).copied().unwrap_or(false)
                                };
                                let weapon = Memo::new(move |_| {
                                    state.get().results.and_then(|r| r.get(&uid_weapon).cloned())
                                });
                                let last_wins = move || {
                                    state.get().scoreboard.get(&uid_score).map(|(last, _)| *last).unwrap_or(0)
                                };
                                view! {
                                    <div
                                        data-name="JyankenPlayerCard"
                                        class="relative flex w-28 flex-col overflow-hidden rounded-xl border bg-card text-card-foreground shadow-sm sm:w-36"
                                        class:revealed=move || revealed.get() && weapon.get().is_some()
                                    >
                                        <div class="flex min-h-12 flex-1 flex-col items-center justify-center px-6 py-2">
                                            <Button
                                                variant=ButtonVariant::Ghost
                                                size=ButtonSize::IconXs
                                                class="absolute top-1.5 left-1.5 text-muted-foreground"
                                                attr:title="Move to spectators"
                                                attr:aria-label="Move to spectators"
                                                on:click=move |_| {
                                                    room_msg(JyankenRequestMessage::RemoveFromGame {
                                                        target_id: uid_remove.clone(),
                                                    });
                                                }
                                            >
                                                <EyeOff class="size-3.5" />
                                            </Button>
                                            <span class="line-clamp-2 text-center text-sm leading-tight font-medium break-words">{name}</span>
                                            {move || if has_shot() {
                                                view! {
                                                    <Badge variant=BadgeVariant::Default size=BadgeSize::Sm class="absolute top-1.5 right-1.5 size-5 justify-center rounded-full p-0 bg-success text-success-foreground" attr:aria-hidden="true">
                                                        <Check class="size-3" />
                                                    </Badge>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <Badge variant=BadgeVariant::Muted size=BadgeSize::Sm class="absolute top-1.5 right-1.5 size-5 justify-center rounded-full p-0" attr:aria-hidden="true">
                                                        <CircleHelp class="size-3" />
                                                    </Badge>
                                                }.into_any()
                                            }}
                                        </div>
                                        <div class="relative aspect-square overflow-hidden perspective-[1000px]">
                                            <div class="flip-face flip-front size-full">
                                                {move || view! { <Avatar avatar=avatar() /> }}
                                            </div>
                                            <div class="flip-face flip-back absolute inset-0 flex flex-col items-center justify-center gap-1 bg-card">
                                                {move || weapon.get().map(|w| {
                                                    let label = weapon_label(&w);
                                                    view! {
                                                        <WeaponIcon weapon=w class="text-4xl sm:text-6xl" />
                                                        <span class="text-xs font-medium sm:text-sm">{label}</span>
                                                    }
                                                })}
                                                <Badge variant=BadgeVariant::Muted size=BadgeSize::Sm class="absolute bottom-1.5 right-1.5">
                                                    {move || format!("+{}", last_wins())}
                                                </Badge>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }
                        </For>
                    </div>
                    <Show when=move || !scoreboard.get().is_empty()>
                        <section class="w-64 shrink-0 rounded-lg border p-3" aria-label="Scoreboard">
                            <h3 class="mb-2 flex items-center gap-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                                <Trophy class="size-3.5" />
                                "Scoreboard"
                            </h3>
                            <table class="w-full text-sm">
                                <thead class="text-left text-xs text-muted-foreground">
                                    <tr>
                                        <th class="font-medium">"Player"</th>
                                        <th class="text-right font-medium">"Last round"</th>
                                        <th class="text-right font-medium">"Total"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <For
                                        each=move || scoreboard.get()
                                        key=|row| row.clone()
                                        let:row
                                    >
                                        <tr>
                                            <td class="truncate py-0.5">{row.0}</td>
                                            <td class="py-0.5 text-right tabular-nums">{row.1}</td>
                                            <td class="py-0.5 text-right font-semibold tabular-nums">{row.2}</td>
                                        </tr>
                                    </For>
                                </tbody>
                            </table>
                        </section>
                    </Show>
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
                        let pressed = weapon.clone();
                        let shoot = weapon.clone();
                        let label = format!("Shoot {}", weapon_label(&weapon));
                        view! {
                            <Button
                                variant=Signal::derive(move || {
                                    if state.get().own_weapon.as_ref() == Some(&picked) {
                                        ButtonVariant::Success
                                    } else {
                                        ButtonVariant::Outline
                                    }
                                })
                                class="size-14 rounded-full sm:size-16"
                                attr:disabled=move || revealed.get()
                                attr:aria-label=label
                                attr:aria-pressed=move || (state.get().own_weapon.as_ref() == Some(&pressed)).to_string()
                                on:click=move |_| {
                                    room_msg(JyankenRequestMessage::Shoot { weapon: shoot.clone() });
                                }
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
        </Card>
    }
}
