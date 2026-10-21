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
use icons::{Check, CircleHelp, EyeOff};
use leptos::prelude::*;

/// A player's card: name, a "done" badge and a button to move them to the spectators on top,
/// their avatar below. When `revealed`, the avatar flips over to show `children` (their vote,
/// their weapon, ...).
#[component]
pub fn PlayerCard(
    #[prop(into)] name: Signal<String>,
    #[prop(into)] avatar: Signal<String>,
    /// Whether the player already played this round (voted, shot, ...).
    #[prop(into)]
    done: Signal<bool>,
    /// Flip the card over to show its back face.
    #[prop(into)]
    revealed: Signal<bool>,
    /// Highlight the card, e.g. when picked by the randomizer.
    #[prop(into, default = Signal::stored(false))]
    selected: Signal<bool>,
    /// Called when someone moves this player to the spectators.
    on_remove: Callback<()>,
    /// The back face of the card.
    children: Children,
) -> impl IntoView {
    // Only re-create the <img> when the avatar actually changes.
    let avatar = Memo::new(move |_| avatar.get());

    view! {
        <div
            data-name="PlayerCard"
            class="relative flex w-28 flex-col overflow-hidden rounded-xl border bg-card text-card-foreground shadow-sm transition-shadow sm:w-36"
            class=("ring-3", move || selected.get())
            class=("ring-success", move || selected.get())
            class=("shadow-lg", move || selected.get())
            class:revealed=move || revealed.get()
        >
            <div class="flex min-h-12 flex-1 flex-col items-center justify-center px-6 py-2">
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::IconXs
                    class="absolute top-1.5 left-1.5 text-muted-foreground"
                    attr:title="Move to spectators"
                    attr:aria-label="Move to spectators"
                    on:click=move |_| on_remove.run(())
                >
                    <EyeOff class="size-3.5" />
                </Button>
                <span class="line-clamp-2 text-center text-sm leading-tight font-medium break-words">
                    {move || name.get()}
                </span>
                {move || if done.get() {
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
                    {move || view! { <Avatar avatar=avatar.get() /> }}
                </div>
                <div class="flip-face flip-back absolute inset-0 flex flex-col items-center justify-center gap-1 bg-card">
                    {children()}
                </div>
            </div>
        </div>
    }
}
