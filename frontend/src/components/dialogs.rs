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

use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::dialog::{
    Dialog, DialogBody, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::components::ui::input::{Input, InputType};
use crate::components::ui::label::Label;
use leptos::prelude::*;
use sizematters_shared::RoomType;
use strum::IntoEnumIterator;

/// A simple modal dialog with a single text input.
/// Pass `content` as a slice of paragraphs to display above the input.
#[component]
pub fn PromptDialog(
    title: &'static str,
    #[prop(default = &[])] content: &'static [&'static str],
    show: RwSignal<bool>,
    on_confirm: Callback<String>,
) -> impl IntoView {
    let input_value = RwSignal::new(String::new());
    // Unique per dialog, since several PromptDialogs share a page.
    let title_id = format!(
        "prompt-dialog-{}",
        title
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "-")
    );
    let input_id = StoredValue::new(format!("{title_id}-input"));
    let title_id = StoredValue::new(title_id);

    let on_submit = move |e: leptos::ev::SubmitEvent| {
        e.prevent_default();
        let val = input_value.get();
        if !val.is_empty() {
            on_confirm.run(val);
        }
        show.set(false);
        input_value.set(String::new());
    };

    view! {
        <Dialog open=show labelledby=title_id.get_value()>
            <form class="contents" on:submit=on_submit>
                <DialogHeader>
                    <DialogTitle attr:id=title_id.get_value()>{title}</DialogTitle>
                    {(!content.is_empty()).then(|| view! {
                        <DialogDescription>
                            {content.iter().map(|&line| view! { <p>{line}</p> }).collect_view()}
                        </DialogDescription>
                    })}
                </DialogHeader>
                <DialogBody>
                    <Input
                        id=input_id.get_value()
                        bind_value=input_value
                        attr:aria-labelledby=title_id.get_value()
                    />
                </DialogBody>
                <DialogFooter>
                    <Button variant=ButtonVariant::Outline attr:r#type="button" on:click=move |_| show.set(false)>
                        "Cancel"
                    </Button>
                    <Button attr:r#type="submit">"Confirm"</Button>
                </DialogFooter>
            </form>
        </Dialog>
    }
}

/// Which action `RoomDialog`'s form submits as.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum RoomDialogMode {
    #[default]
    Join,
    Create,
}

/// The outcome of confirming `RoomDialog`: either join an existing room, or create one of a
/// given `RoomType`.
#[derive(Clone)]
pub enum RoomDialogAction {
    Join {
        room_name: String,
        password: String,
    },
    Create {
        room_name: String,
        password: String,
        room_type: RoomType,
    },
}

/// Modal dialog for creating/joining a rooms with name and password fields.
#[component]
pub fn RoomDialog(show: RwSignal<bool>, on_confirm: Callback<RoomDialogAction>) -> impl IntoView {
    let mode = RwSignal::new(RoomDialogMode::default());
    let room_name = RwSignal::new(String::new());
    let room_password = RwSignal::new(String::new());
    let room_type = RwSignal::new(RoomType::iter().next().expect("RoomType has no variants"));

    let on_submit = move |e: leptos::ev::SubmitEvent| {
        e.prevent_default();
        let name = room_name.get();
        let password = room_password.get();
        if !name.is_empty() {
            let action = match mode.get() {
                RoomDialogMode::Join => RoomDialogAction::Join {
                    room_name: name,
                    password,
                },
                RoomDialogMode::Create => RoomDialogAction::Create {
                    room_name: name,
                    password,
                    room_type: room_type.get(),
                },
            };
            on_confirm.run(action);
        }
        show.set(false);
        room_name.set(String::new());
        room_password.set(String::new());
    };

    view! {
        <Dialog open=show labelledby="room-dialog-title">
            <form class="contents" on:submit=on_submit>
                <DialogHeader>
                    <DialogTitle attr:id="room-dialog-title">"Create/Join Room"</DialogTitle>
                    <DialogDescription>
                        <p>
                            "If someone gave you a rooms name/password combination, just enter it below to join that rooms."
                        </p>
                        <p>
                            "Or type a new name and select a password to create your own rooms."
                            <br />
                            "Then share it with your cow-orkers in order to have a size battle."
                        </p>
                    </DialogDescription>
                </DialogHeader>
                <DialogBody>
                    <div class="flex gap-4" role="radiogroup" aria-label="Create or join a room">
                        <Label class="font-normal">
                            <input
                                r#type="radio"
                                name="room-dialog-mode"
                                checked=move || mode.get() == RoomDialogMode::Create
                                on:change=move |_| mode.set(RoomDialogMode::Create)
                            />
                            "Create room"
                        </Label>
                        <Label class="font-normal">
                            <input
                                r#type="radio"
                                name="room-dialog-mode"
                                checked=move || mode.get() == RoomDialogMode::Join
                                on:change=move |_| mode.set(RoomDialogMode::Join)
                            />
                            "Join room"
                        </Label>
                    </div>
                    <div class="flex flex-col gap-2">
                        <Label html_for="room-name-input">"Room Name"</Label>
                        <Input id="room-name-input" autocomplete="off" bind_value=room_name />
                    </div>
                    <div class="flex flex-col gap-2">
                        <Label html_for="room-password-input">"Room Password"</Label>
                        <Input
                            id="room-password-input"
                            r#type=InputType::Password
                            autocomplete="off"
                            bind_value=room_password
                        />
                    </div>
                    <Show when=move || mode.get() == RoomDialogMode::Create>
                        <div class="flex flex-col gap-2">
                            <Label html_for="room-type-select">"Room Type"</Label>
                            <select
                                id="room-type-select"
                                class="text-foreground border-input flex h-9 w-full min-w-0 rounded-md border bg-transparent px-3 py-1 text-base shadow-xs outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-2 md:text-sm"
                                on:change=move |e| {
                                    let index: usize = event_target_value(&e).parse().unwrap_or(0);
                                    if let Some(selected) = RoomType::iter().nth(index) {
                                        room_type.set(selected);
                                    }
                                }
                            >
                                {RoomType::iter()
                                    .enumerate()
                                    .map(|(index, rt)| {
                                        view! {
                                            <option value=index.to_string()>{rt.display_name()}</option>
                                        }
                                    })
                                    .collect_view()}
                            </select>
                        </div>
                    </Show>
                </DialogBody>
                <DialogFooter>
                    <Button variant=ButtonVariant::Outline attr:r#type="button" on:click=move |_| show.set(false)>
                        "Close"
                    </Button>
                    <Button attr:r#type="submit">
                        {move || if mode.get() == RoomDialogMode::Create { "Create" } else { "Join" }}
                    </Button>
                </DialogFooter>
            </form>
        </Dialog>
    }
}
