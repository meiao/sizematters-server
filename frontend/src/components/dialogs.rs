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

use leptos::prelude::*;
use web_sys::HtmlInputElement;

/// A simple modal dialog with a single text input.
/// Pass `content` as a slice of paragraphs to display above the input.
#[component]
pub fn PromptDialog(
    title: &'static str,
    #[prop(default = &[])] content: &'static [&'static str],
    show: RwSignal<bool>,
    #[prop(optional, into)] initial_value: Option<Signal<String>>,
    on_confirm: Callback<String>,
) -> impl IntoView {
    let input_value = RwSignal::new(String::new());
    let input_ref: NodeRef<leptos::html::Input> = NodeRef::new();

    // Populate initial value and focus when dialog opens
    Effect::new(move |_| {
        if show.get() {
            if let Some(iv) = initial_value {
                input_value.set(iv.get_untracked());
            }
            request_animation_frame(move || {
                if let Some(el) = input_ref.get() {
                    let _ = el.focus();
                    let _ = el.select();
                }
            });
        }
    });

    let on_submit = move || {
        let val = input_value.get();
        if !val.is_empty() {
            on_confirm.run(val);
        }
        show.set(false);
        input_value.set(String::new());
    };

    view! {
        <Show when=move || show.get()>
            <div
                class="dialog-overlay"
                on:click=move |_| show.set(false)
                on:keydown=move |e: web_sys::KeyboardEvent| {
                    if e.key() == "Escape" {
                        show.set(false);
                    }
                }
            >
                <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="prompt-dialog-title" on:click=|e| e.stop_propagation()>
                    <div class="dialog-title" id="prompt-dialog-title">{title}</div>
                    {(!content.is_empty()).then(|| view! {
                        <div class="dialog-content">
                            {content.iter().map(|&line| view! { <p>{line}</p> }).collect_view()}
                        </div>
                    })}
                    <div class="dialog-body">
                        <input
                            type="text"
                            class="dialog-input"
                            aria-labelledby="prompt-dialog-title"
                            node_ref=input_ref
                            prop:value=move || input_value.get()
                            on:input=move |e| {
                                let target: HtmlInputElement = event_target(&e);
                                input_value.set(target.value());
                            }
                            on:keydown=move |e: web_sys::KeyboardEvent| {
                                if e.key() == "Enter" {
                                    on_submit();
                                }
                            }
                        />
                    </div>
                    <div class="dialog-actions">
                        <button class="btn" on:click=move |_| show.set(false)>"Cancel"</button>
                        <button class="btn btn-primary" on:click=move |_| on_submit()>"Confirm"</button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// Modal dialog for creating/joining a room with name and password fields.
#[component]
pub fn RoomDialog(
    show: RwSignal<bool>,
    on_confirm: Callback<(String, String)>,
) -> impl IntoView {
    let room_name = RwSignal::new(String::new());
    let room_password = RwSignal::new(String::new());
    let room_name_ref: NodeRef<leptos::html::Input> = NodeRef::new();

    Effect::new(move |_| {
        if show.get() {
            request_animation_frame(move || {
                if let Some(el) = room_name_ref.get() {
                    let _ = el.focus();
                }
            });
        }
    });

    let on_submit = move || {
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
        <Show when=move || show.get()>
            <div
                class="dialog-overlay"
                on:click=move |_| show.set(false)
                on:keydown=move |e: web_sys::KeyboardEvent| {
                    if e.key() == "Escape" {
                        show.set(false);
                    }
                }
            >
                <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="room-dialog-title" on:click=|e| e.stop_propagation()>
                    <div class="dialog-title" id="room-dialog-title">"Create/Join Room"</div>
                    <div class="dialog-content">
                        <p>
                            "If someone gave you a room name/password combination, just enter it below to join that room."
                        </p>
                        <p>
                            "Or type a new name and select a password to create your own room."
                            <br />
                            "Then share it with your cow-orkers in order to have a size battle."
                        </p>
                    </div>
                    <div class="dialog-body">
                        <div class="field">
                            <label for="room-name-input">"Room Name"</label>
                            <input
                                id="room-name-input"
                                type="text"
                                class="dialog-input"
                                node_ref=room_name_ref
                                prop:value=move || room_name.get()
                                on:input=move |e| {
                                    let target: HtmlInputElement = event_target(&e);
                                    room_name.set(target.value());
                                }
                                on:keydown=move |e: web_sys::KeyboardEvent| {
                                    if e.key() == "Enter" {
                                        on_submit();
                                    }
                                }
                            />
                        </div>
                        <div class="field">
                            <label for="room-password-input">"Room Password"</label>
                            <input
                                id="room-password-input"
                                type="password"
                                class="dialog-input"
                                prop:value=move || room_password.get()
                                on:input=move |e| {
                                    let target: HtmlInputElement = event_target(&e);
                                    room_password.set(target.value());
                                }
                                on:keydown=move |e: web_sys::KeyboardEvent| {
                                    if e.key() == "Enter" {
                                        on_submit();
                                    }
                                }
                            />
                        </div>
                    </div>
                    <div class="dialog-actions">
                        <button class="btn" on:click=move |_| show.set(false)>"Close"</button>
                        <button class="btn btn-primary btn-raised" on:click=move |_| on_submit()>"Join"</button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
