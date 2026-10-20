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

use super::link::room_link;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardAction, CardDescription, CardHeader, CardTitle};
use crate::stores::RoomStore;
use icons::{Check, Link};
use leptos::prelude::*;

/// The frame of a room: a card whose header shows the room name, its shareable link and a
/// button to copy that link. The room type's content (usually a `CardContent` and a
/// `CardFooter`) goes in `children`.
#[component]
pub fn RoomShell(room_name: String, children: Children) -> impl IntoView {
    let room_store = expect_context::<RoomStore>();
    let room_link = room_link(room_store, room_name.clone());
    let link_copied = RwSignal::new(false);

    let copy_link = move |_| {
        let link = room_link.get_untracked();
        if let Some(window) = web_sys::window() {
            let _ = window.navigator().clipboard().write_text(&link);
            link_copied.set(true);
            set_timeout(
                move || link_copied.set(false),
                std::time::Duration::from_secs(2),
            );
        }
    };

    view! {
        <Card class="gap-0 overflow-hidden pt-0">
            <CardHeader class="grid grid-cols-[minmax(0,1fr)_auto] bg-black py-4 text-white dark:border-b dark:border-white/10">
                <CardTitle class="text-lg">{room_name}</CardTitle>
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
                        on:click=copy_link
                    >
                        {move || if link_copied.get() {
                            view! { <Check /> }.into_any()
                        } else {
                            view! { <Link /> }.into_any()
                        }}
                    </Button>
                </CardAction>
            </CardHeader>
            {children()}
        </Card>
    }
}
