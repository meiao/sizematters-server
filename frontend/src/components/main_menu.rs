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

use crate::components::dialogs::{PromptDialog, RoomDialog, RoomDialogAction};
use crate::components::ui::avatar::{Avatar, AvatarFallback, AvatarImage, AvatarSize};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{
    Card, CardAction, CardDescription, CardHeader, CardSize, CardTitle,
};
use crate::stores::{RoomStore, UserStore, VoteStore};
use crate::ws;
use icons::{Plus, UserPen};
use leptos::prelude::*;

#[component]
pub fn MainMenu() -> impl IntoView {
    let user_store = expect_context::<UserStore>();
    let room_store = expect_context::<RoomStore>();
    let vote_store = expect_context::<VoteStore>();

    let show_name_dialog = RwSignal::new(false);
    let show_email_dialog = RwSignal::new(false);
    let show_room_dialog = RwSignal::new(false);

    let name = Memo::new(move |_| {
        let own_id = user_store.own_user_id();
        let users = user_store.user_signal().get();
        users
            .get(&own_id)
            .map(|u| u.name.clone())
            .unwrap_or("not connected".to_string())
    });

    let img_url = Memo::new(move |_| {
        let own_id = user_store.own_user_id();
        let users = user_store.user_signal().get();
        users
            .get(&own_id)
            .map(|u| format!("https://www.gravatar.com/avatar/{}?d=retro", u.gravatar_id))
            .unwrap_or_default()
    });

    let rooms = Memo::new(move |_| room_store.rooms_signal().get());

    // Register on creation
    ws::ws_register();

    view! {
        <PromptDialog
            title="What's your name?"
            show=show_name_dialog
            on_confirm=Callback::new(move |name: String| {
                ws::ws_set_name(name);
            })
        />
        <PromptDialog
            title="Profile picture"
            content=&[
                "I tried my best to create a profile picture for you. Since you didn't like it, you can use your Gravatar image.",
                "I will need your email for such. I promise I won't save/sell your email.",
            ]
            show=show_email_dialog
            on_confirm=Callback::new(|email: String| {
                ws::ws_set_avatar(email);
            })
        />
        <RoomDialog
            show=show_room_dialog
            on_confirm=Callback::new(|action: RoomDialogAction| match action {
                RoomDialogAction::Join { room_name, password } => {
                    ws::ws_join_room(room_name, password, false);
                }
                RoomDialogAction::Create { room_name, password, room_type } => {
                    ws::ws_create_room(room_name, password, false, room_type);
                }
            })
        />

        <Card size=CardSize::Sm class="flex-row items-center gap-1 px-2">
            <Button
                variant=ButtonVariant::Ghost
                size=ButtonSize::Icon
                class="size-12 rounded-full p-0"
                attr:aria-label="Change profile picture"
                attr:title="Change profile picture"
                on:click=move |_| show_email_dialog.set(true)
            >
                <Avatar size=AvatarSize::Lg class="size-11">
                    {move || {
                        let url = img_url.get();
                        (!url.is_empty()).then(|| view! { <AvatarImage attr:src=url attr:alt="" /> })
                    }}
                    <AvatarFallback class="bg-success text-success-foreground text-lg">
                        {move || name.get().chars().next().map(String::from).unwrap_or_default()}
                    </AvatarFallback>
                </Avatar>
            </Button>
            <Button
                variant=ButtonVariant::Ghost
                class="min-w-0 flex-1 justify-between text-base"
                attr:aria-label="Change your name"
                attr:title="Change your name"
                on:click=move |_| show_name_dialog.set(true)
            >
                <span class="truncate">{move || name.get()}</span>
                <UserPen class="text-muted-foreground" />
            </Button>
        </Card>

        <section class="flex flex-col gap-3" aria-labelledby="menu-room-title">
            <header class="flex items-center justify-between">
                <h2 id="menu-room-title" class="text-sm font-semibold text-muted-foreground uppercase tracking-wide">
                    "Rooms"
                </h2>
                <Button
                    variant=ButtonVariant::Outline
                    size=ButtonSize::Icon
                    class="rounded-full"
                    attr:aria-label="Create or join a room"
                    attr:title="Create or join a room"
                    on:click=move |_| show_room_dialog.set(true)
                >
                    <Plus />
                </Button>
            </header>
            <Show when=move || rooms.get().is_empty()>
                <p class="rounded-lg border border-dashed px-3 py-2 text-right text-sm text-muted-foreground">
                    "Yes, the \"+\" over here ↑"
                </p>
            </Show>
            <For
                each=move || rooms.get()
                key=|room| room.room_name.clone()
                let:room
            >
                {
                    let rn = room.room_name.clone();
                    let rn2 = room.room_name.clone();
                    view! {
                        <Card size=CardSize::Sm>
                            <CardHeader class="grid grid-cols-[minmax(0,1fr)_auto]">
                                <CardTitle class="truncate text-base">{rn}</CardTitle>
                                <CardDescription>
                                    "Voting: " {room.votes_cast} "/" {room.users.len().saturating_sub(vote_store.spectator_count(&room.room_name))}
                                </CardDescription>
                                <CardAction class="col-start-2 row-span-2 row-start-1 self-center">
                                    <Button
                                        variant=ButtonVariant::Outline
                                        size=ButtonSize::Sm
                                        on:click=move |_| {
                                            ws::ws_leave_room(rn2.clone());
                                            room_store.leave_room(&rn2);
                                            vote_store.leave_room(&rn2);
                                        }
                                    >
                                        "Leave"
                                    </Button>
                                </CardAction>
                            </CardHeader>
                        </Card>
                    }
                }
            </For>
        </section>
    }
}
