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

use crate::components::gravatar::Gravatar;
use crate::components::ui::badge::{Badge, BadgeSize, BadgeVariant};
use crate::stores::{UserStore, VoteStore};
use icons::{Check, CircleHelp};
use leptos::prelude::*;

#[component]
pub fn UserCard(
    user_id: String,
    room_name: String,
    /// Everyone has voted, so cards flip over to show the vote.
    #[prop(into)]
    revealed: Signal<bool>,
    /// Picked by the randomizer.
    #[prop(into)]
    selected: Signal<bool>,
) -> impl IntoView {
    let user_store = expect_context::<UserStore>();
    let vote_store = expect_context::<VoteStore>();

    let uid = user_id.clone();
    let user_data = Memo::new(move |_| {
        let users = user_store.user_signal().get();
        users.get(&uid).cloned()
    });

    let uid2 = user_id.clone();
    let rn2 = room_name.clone();
    let user_vote = Memo::new(move |_| {
        let votes = vote_store.votes_signal().get();
        votes.get(&rn2).and_then(|rv| rv.get(&uid2)).cloned()
    });

    let name = move || user_data.get().map(|u| u.name.clone()).unwrap_or_default();

    // Memo for gravatar_id so the <img> only re-creates when it actually changes.
    let gravatar_id = Memo::new(move |_| {
        user_data
            .get()
            .map(|u| u.gravatar_id.clone())
            .unwrap_or_default()
    });

    let has_voted = move || user_vote.get().map(|v| v.has_voted).unwrap_or(false);

    let vote_value = move || {
        user_vote
            .get()
            .and_then(|v| v.value)
            .map(|v| v.to_string())
            .unwrap_or_default()
    };

    let order = move || calculate_order(&name());

    view! {
        <div
            data-name="UserCard"
            class="relative flex w-28 flex-col overflow-hidden rounded-xl border bg-card text-card-foreground shadow-sm transition-shadow sm:w-36"
            class=("ring-3", move || selected.get())
            class=("ring-success", move || selected.get())
            class=("shadow-lg", move || selected.get())
            class:revealed=move || revealed.get() && has_voted()
            style:order=order
        >
            <div class="flex min-h-12 flex-1 flex-col items-center justify-center px-6 py-2">
                <span class="line-clamp-2 text-center text-sm leading-tight font-medium break-words">{name}</span>
                {move || if has_voted() {
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
                    {move || {
                        let gid = gravatar_id.get();
                        view! { <Gravatar gravatar_id=gid /> }
                    }}
                </div>
                <div class="flip-face flip-back absolute inset-0 flex items-center justify-center bg-card text-4xl font-semibold sm:text-6xl">
                    {vote_value}
                </div>
            </div>
        </div>
    }
}

/// Compute a CSS `order` value for alphabetical sorting of user cards.
/// Uses Unicode scalar values, clamped to avoid overflow.
fn calculate_order(name: &str) -> String {
    let lower = name.to_lowercase();
    let chars: Vec<char> = lower.chars().take(4).collect();
    let padded: Vec<u32> = (0..4)
        .map(|i| {
            if i < chars.len() {
                // Clamp to 2 digits: 'a'=10, 'z'=35, non-alpha gets 10
                let c = chars[i] as u32;
                let a = 'a' as u32;
                if c >= a && c <= a + 25 {
                    c - a + 10
                } else {
                    10
                }
            } else {
                10
            }
        })
        .collect();
    padded
        .iter()
        .map(|n| format!("{:02}", n))
        .collect::<String>()
}
