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

use super::store::JyankenState;
use super::weapon::{weapon_label, WeaponIcon};
use crate::components::ui::badge::{Badge, BadgeSize, BadgeVariant};
use crate::rooms::common::{send_room_msg, PlayerCard};
use leptos::prelude::*;
use sizematters_shared::messages::jyanken::JyankenRequestMessage;
use sizematters_shared::UserData;

/// A player's card. Shows whether they shot, and flips over to show their weapon and the
/// wins of the round once everyone shot.
#[component]
pub fn JyankenCard(
    user_id: String,
    room_name: String,
    /// The users in the room.
    #[prop(into)]
    users: Signal<Vec<UserData>>,
    /// The state of the room.
    #[prop(into)]
    state: Signal<JyankenState>,
) -> impl IntoView {
    let uid = user_id.clone();
    let user =
        Memo::new(move |_| users.with(|users| users.iter().find(|u| u.user_id == uid).cloned()));

    let uid = user_id.clone();
    let has_shot =
        Signal::derive(move || state.with(|s| s.selected.get(&uid).copied().unwrap_or(false)));

    let uid = user_id.clone();
    let weapon =
        Memo::new(move |_| state.with(|s| s.results.as_ref().and_then(|r| r.get(&uid).cloned())));

    let uid = user_id.clone();
    let last_wins =
        move || state.with(|s| s.scoreboard.get(&uid).map(|(last, _)| *last).unwrap_or(0));

    let remove = Callback::new(move |_| {
        send_room_msg(
            &room_name,
            &JyankenRequestMessage::RemoveFromGame {
                target_id: user_id.clone(),
            },
        );
    });

    view! {
        <PlayerCard
            name=Signal::derive(move || user.get().map(|u| u.name).unwrap_or_default())
            avatar=Signal::derive(move || user.get().map(|u| u.avatar).unwrap_or_default())
            done=has_shot
            revealed=Signal::derive(move || weapon.with(Option::is_some))
            on_remove=remove
        >
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
        </PlayerCard>
    }
}
