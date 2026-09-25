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

use crate::components::rooms::sizing::Room;
use crate::components::ui::alert::{Alert, AlertDescription, AlertTitle};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::empty::{
    Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
use crate::stores::{RoomStore, UserStore};
use crate::ws::WsContext;
use icons::{ArrowLeft, CircleAlert, Sparkles, X};
use leptos::prelude::*;

#[component]
pub fn MainPage() -> impl IntoView {
    let user_store = expect_context::<UserStore>();
    let room_store = expect_context::<RoomStore>();
    let ws_ctx = expect_context::<WsContext>();

    let is_default_name = Memo::new(move |_| {
        let own_id = user_store.own_user_id();
        user_store.user_signal().with(|users| {
            users
                .get(&own_id)
                .map(|u| u.name == "Shirtless Muppet")
                .unwrap_or(false)
        })
    });

    let banner_dismissed = RwSignal::new(false);
    let rooms = Memo::new(move |_| room_store.rooms_signal().get());
    let last_error = ws_ctx.last_error();

    let dismiss_banner = move |_| {
        banner_dismissed.set(true);
    };

    view! {
        <div class="flex flex-col gap-4" on:click=dismiss_banner>
            <div class="pointer-events-none absolute inset-x-4 top-4 z-10 flex flex-col items-end gap-2 md:inset-x-6">
                // Error toast from server messages, floating over the rooms
                <Show when=move || last_error.get().is_some()>
                    <Alert
                        class="pointer-events-auto w-full max-w-md border-destructive/50 bg-card pr-12 text-destructive shadow-lg"
                        attr:role="alert"
                    >
                        <CircleAlert class="size-4 text-destructive" />
                        <AlertTitle>"Something went wrong"</AlertTitle>
                        <AlertDescription>{move || last_error.get().unwrap_or_default()}</AlertDescription>
                        <Button
                            variant=ButtonVariant::Ghost
                            size=ButtonSize::IconSm
                            class="absolute top-2 right-2 text-destructive hover:text-destructive"
                            attr:aria-label="Dismiss error"
                            on:click=move |e| {
                                e.stop_propagation();
                                ws_ctx.clear_error();
                            }
                        >
                            <X />
                        </Button>
                    </Alert>
                </Show>
            </div>

            // Banner prompting new users to set their name
            <Show when=move || is_default_name.get() && !banner_dismissed.get()>
                <Alert class="w-fit max-w-md bg-card" attr:role="status">
                    <ArrowLeft class="size-4" />
                    <AlertTitle>"Hi " <b>"Shirtless Muppet"</b> "."</AlertTitle>
                    <AlertDescription class="text-muted-foreground">
                        "You can change your name by clicking it in the menu."
                    </AlertDescription>
                </Alert>
            </Show>

            <For
                each=move || rooms.get()
                key=|room| room.room_name.clone()
                let:room
            >
                {
                    let rn = room.room_name.clone();
                    view! { <Room room_name=rn /> }
                }
            </For>

            <Show when=move || rooms.get().is_empty()>
                <Empty class="mt-12">
                    <EmptyHeader>
                        <EmptyMedia variant=EmptyMediaVariant::Icon>
                            <Sparkles />
                        </EmptyMedia>
                        <EmptyTitle>"Join a room"</EmptyTitle>
                        <EmptyDescription class="max-w-sm">
                            "To get the most of this website you should join a room, or create one. Do so by pressing the '+' in the menu."
                        </EmptyDescription>
                    </EmptyHeader>
                </Empty>
            </Show>
        </div>
    }
}
