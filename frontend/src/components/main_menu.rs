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

use crate::components::dialogs::{RoomDialog, RoomDialogAction, Settings, SettingsDialog};
use crate::components::ui::avatar::{Avatar, AvatarFallback, AvatarImage, AvatarSize};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::{Card, CardAction, CardHeader, CardSize, CardTitle};
use crate::room_storage;
use crate::rooms::RoomStores;
use crate::stores::{RoomStore, UserStore};
use crate::ws;
use icons::{Plus, Settings as SettingsIcon};
use leptos::prelude::*;

#[component]
pub fn MainMenu() -> impl IntoView {
    let user_store = expect_context::<UserStore>();
    let room_store = expect_context::<RoomStore>();
    let rooms_stores = expect_context::<RoomStores>();

    let show_settings_dialog = RwSignal::new(false);
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
            .map(|u| u.avatar.clone())
            .unwrap_or_default()
    });

    let rooms = Memo::new(move |_| room_store.rooms_signal().get());

    // Register on creation
    ws::ws_register();

    view! {
        <SettingsDialog
            show=show_settings_dialog
            current_name=Signal::from(name)
            on_confirm=Callback::new(move |settings: Settings| {
                if !settings.name.is_empty() && settings.name != name.get_untracked() {
                    ws::ws_set_name(settings.name);
                }
                ws::ws_set_avatar_style(settings.avatar_style);
                room_storage::set_randomizer_muted(!settings.sound_on);
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
                // Same behaviour as loading the URL (see `ConnectingPage`): create or join
                // when the type is known, otherwise join only. The password is already hashed.
                RoomDialogAction::Url(room_url) => match room_url.room_type {
                    Some(room_type) => ws::ws_create_room(
                        room_url.room_name,
                        room_url.hashed_password,
                        true,
                        room_type,
                    ),
                    None => ws::ws_join_room(room_url.room_name, room_url.hashed_password, true),
                },
            })
        />

        <Card size=CardSize::Sm class="flex-row items-center gap-1 px-2">
            <Avatar size=AvatarSize::Lg class="size-11 shrink-0">
                {move || {
                    let url = img_url.get();
                    (!url.is_empty()).then(|| view! { <AvatarImage attr:src=url attr:alt="" /> })
                }}
                {move || {
                    img_url.get().is_empty().then(|| view! {
                        <AvatarFallback class="bg-success text-success-foreground text-lg">
                            {move || name.get().chars().next().map(String::from).unwrap_or_default()}
                        </AvatarFallback>
                    })
                }}
            </Avatar>
            <span class="min-w-0 flex-1 truncate px-2 text-base font-medium">{move || name.get()}</span>
            <Button
                variant=ButtonVariant::Ghost
                size=ButtonSize::Icon
                attr:aria-label="Settings"
                attr:title="Settings"
                on:click=move |_| show_settings_dialog.set(true)
            >
                <SettingsIcon class="text-muted-foreground" />
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
                                <CardAction class="col-start-2 row-start-1 self-center">
                                    <Button
                                        variant=ButtonVariant::Outline
                                        size=ButtonSize::Sm
                                        on:click=move |_| {
                                            ws::ws_leave_room(rn2.clone());
                                            room_store.leave_room(&rn2);
                                            rooms_stores.leave(&rn2);
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
