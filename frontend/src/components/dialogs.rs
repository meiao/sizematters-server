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
    let title_id = format!("prompt-dialog-{}", title.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "-"));
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

/// Modal dialog for creating/joining a room with name and password fields.
#[component]
pub fn RoomDialog(show: RwSignal<bool>, on_confirm: Callback<(String, String)>) -> impl IntoView {
    let room_name = RwSignal::new(String::new());
    let room_password = RwSignal::new(String::new());

    let on_submit = move |e: leptos::ev::SubmitEvent| {
        e.prevent_default();
        let name = room_name.get();
        let pw = room_password.get();
        if !name.is_empty() {
            on_confirm.run((name, pw));
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
                            "If someone gave you a room name/password combination, just enter it below to join that room."
                        </p>
                        <p>
                            "Or type a new name and select a password to create your own room."
                            <br />
                            "Then share it with your cow-orkers in order to have a size battle."
                        </p>
                    </DialogDescription>
                </DialogHeader>
                <DialogBody>
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
                </DialogBody>
                <DialogFooter>
                    <Button variant=ButtonVariant::Outline attr:r#type="button" on:click=move |_| show.set(false)>
                        "Close"
                    </Button>
                    <Button attr:r#type="submit">"Join"</Button>
                </DialogFooter>
            </form>
        </Dialog>
    }
}
