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

use crate::room_storage;
use crate::rooms::{HandlerCtx, RoomStores};
use crate::stores::{RoomStore, UserStore};
use leptos::prelude::*;
use sizematters_shared::avatar::AvatarStyle;
use sizematters_shared::messages::{ClientRequestMessage, ClientResponseMessage};
use sizematters_shared::RoomType;
use wasm_bindgen::prelude::*;
use web_sys::WebSocket;

/// Global WebSocket context provided to all components.
#[derive(Clone, Copy)]
pub struct WsContext {
    connected: RwSignal<bool>,
    /// Last error message from the server, displayed to the user.
    last_error: RwSignal<Option<String>>,
    /// Whether the user has explicitly initiated a connection (enables auto-reconnect).
    initiated: RwSignal<bool>,
    /// Whether a Register message has been sent this session.
    registered: RwSignal<bool>,
    room_store: RoomStore,
    user_store: UserStore,
    rooms: RoomStores,
}

impl WsContext {
    pub fn new(room_store: RoomStore, user_store: UserStore, rooms: RoomStores) -> Self {
        Self {
            connected: RwSignal::new(false),
            last_error: RwSignal::new(None),
            initiated: RwSignal::new(false),
            registered: RwSignal::new(false),
            room_store,
            user_store,
            rooms,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected.get()
    }

    /// Whether the user has explicitly initiated a connection (via `connect()`).
    /// Stable across the sidebar swapping between `NoMenu` and `MainMenu`, so it's
    /// safe to react to from a component that outlives that swap.
    pub fn is_initiated(&self) -> bool {
        self.initiated.get()
    }

    pub fn last_error(&self) -> ReadSignal<Option<String>> {
        self.last_error.read_only()
    }

    pub fn clear_error(&self) {
        self.last_error.set(None);
    }

    /// Open a WebSocket connection. Guards against duplicate calls, including calls made
    /// while a previous attempt is still connecting.
    pub fn connect(&self) {
        self.initiated.set(true);
        install_wake_listeners(*self);

        // Prevent duplicate connections
        let already_connected = SOCKET.with(|s| s.borrow().is_some());
        if already_connected || self.connected.get_untracked() || CONNECTING.get() {
            return;
        }
        CONNECTING.set(true);
        // This attempt supersedes any scheduled one, so there is only ever one retry chain.
        cancel_scheduled_reconnect();

        let location = web_sys::window().unwrap().location();
        let protocol = location.protocol().unwrap();
        let host = location.host().unwrap();
        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
        let url = format!("{}//{}/ws", ws_protocol, host);

        let socket = match WebSocket::new(&url) {
            Ok(s) => s,
            Err(err) => {
                log::error!("Failed to create WebSocket: {:?}", err);
                CONNECTING.set(false);
                self.last_error
                    .set(Some("Failed to connect to server.".to_string()));
                schedule_reconnect(*self);
                return;
            }
        };
        socket.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let connected = self.connected;
        let registered = self.registered;
        let last_error = self.last_error;
        let room_store = self.room_store;
        let user_store = self.user_store;
        let rooms = self.rooms;

        let socket_clone = socket.clone();

        // onopen
        let onopen = Closure::<dyn Fn()>::new(move || {
            log::info!("WebSocket connected");
            CONNECTING.set(false);
            connected.set(true);
            registered.set(false);
            RECONNECT_ATTEMPTS.with(|a| a.set(0));
            SOCKET.with(|s| {
                *s.borrow_mut() = Some(socket_clone.clone());
            });
            // The server treats every socket as a new user, with a new id, so whatever we
            // knew about our rooms (members, votes, spectators) is stale after a reconnect.
            // Drop it and let restore_data() rejoin the rooms and rebuild it. Rooms that
            // can't be rejoined (e.g. the password changed) no longer linger in the UI.
            room_store.clear();
            rooms.clear();
            restore_data();
        });
        socket.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        // onmessage
        let onmessage =
            Closure::<dyn Fn(web_sys::MessageEvent)>::new(move |e: web_sys::MessageEvent| {
                if let Some(text) = e.data().as_string() {
                    match serde_json::from_str::<ClientResponseMessage>(&text) {
                        Ok(msg) => process_message(msg, room_store, user_store, rooms, last_error),
                        Err(err) => log::warn!("Failed to parse message: {}", err),
                    }
                }
            });
        socket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        // onclose — trigger auto-reconnect if the user previously initiated a connection
        let ctx = *self;
        let onclose = Closure::<dyn Fn()>::new(move || {
            log::info!("WebSocket disconnected");
            CONNECTING.set(false);
            ctx.connected.set(false);
            SOCKET.with(|s| {
                *s.borrow_mut() = None;
            });
            schedule_reconnect(ctx);
        });
        socket.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

        // onerror
        let onerror = Closure::<dyn Fn()>::new(move || {
            log::error!("WebSocket error");
        });
        socket.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();
    }
}

thread_local! {
    /// Whether a socket has been created but has not opened (or failed) yet.
    static CONNECTING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static SOCKET: std::cell::RefCell<Option<WebSocket>> = const { std::cell::RefCell::new(None) };
    static RECONNECT_ATTEMPTS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// The pending reconnection timer, if one is scheduled.
    static RECONNECT_TIMER: std::cell::Cell<Option<TimeoutHandle>> = const { std::cell::Cell::new(None) };
    /// Whether the `online`/`visibilitychange` listeners have been installed.
    static WAKE_LISTENERS_INSTALLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Upper bound for the delay between reconnection attempts.
const MAX_RECONNECT_DELAY_MS: u64 = 30_000;

/// Schedule an automatic reconnection with exponential backoff: 1s, 2s, 4s, 8s, 16s, then
/// every 30s for as long as it takes. There is no attempt limit, so the app recovers on its
/// own after e.g. a laptop sleeps or the server is redeployed.
fn schedule_reconnect(ctx: WsContext) {
    if !ctx.initiated.get_untracked() {
        return;
    }

    let attempts = RECONNECT_ATTEMPTS.get();
    RECONNECT_ATTEMPTS.set(attempts.saturating_add(1));

    let delay_ms = (1000u64 << attempts.min(5)).min(MAX_RECONNECT_DELAY_MS);
    log::info!("Reconnecting in {}ms (attempt {})", delay_ms, attempts + 1);

    cancel_scheduled_reconnect();
    let handle = set_timeout_with_handle(
        move || {
            RECONNECT_TIMER.set(None);
            if !ctx.connected.get_untracked() {
                ctx.connect();
            }
        },
        std::time::Duration::from_millis(delay_ms),
    );
    match handle {
        Ok(handle) => RECONNECT_TIMER.set(Some(handle)),
        Err(err) => log::error!("Failed to schedule reconnection: {:?}", err),
    }
}

fn cancel_scheduled_reconnect() {
    if let Some(handle) = RECONNECT_TIMER.take() {
        handle.clear();
    }
}

/// Reconnect right away, skipping the backoff, when the browser comes back online or the
/// tab becomes visible again (e.g. after the laptop wakes up). Installed once.
fn install_wake_listeners(ctx: WsContext) {
    if WAKE_LISTENERS_INSTALLED.replace(true) {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };

    let wake = Closure::<dyn Fn()>::new(move || {
        let visible = web_sys::window()
            .and_then(|w| w.document())
            .map(|d| d.visibility_state() == web_sys::VisibilityState::Visible)
            .unwrap_or(true);
        if visible && ctx.initiated.get_untracked() && !ctx.connected.get_untracked() {
            log::info!("Woke up while disconnected, reconnecting now");
            RECONNECT_ATTEMPTS.set(0);
            ctx.connect();
        }
    });
    let callback = wake.as_ref().unchecked_ref();
    let _ = window.add_event_listener_with_callback("online", callback);
    if let Some(document) = window.document() {
        let _ = document.add_event_listener_with_callback("visibilitychange", callback);
    }
    wake.forget();
}

fn send_message(msg: ClientRequestMessage) {
    SOCKET.with(|s| {
        if let Some(socket) = s.borrow().as_ref() {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    if let Err(err) = socket.send_with_str(&json) {
                        log::warn!("WebSocket send failed: {:?}", err);
                    }
                }
                Err(err) => log::warn!("Failed to serialize message: {}", err),
            }
        } else {
            log::warn!("Cannot send message: WebSocket not connected");
        }
    });
}

/// Restore user name, avatar style and rooms from localStorage after (re)connecting.
/// Does nothing when localStorage is unavailable.
fn restore_data() {
    let Some(storage) = room_storage::local_storage() else {
        return;
    };
    if let Ok(Some(name)) = storage.get_item("name") {
        send_message(ClientRequestMessage::SetName { name });
    }
    if let Ok(Some(saved)) = storage.get_item("avatar_style") {
        if let Ok(avatar_style) = serde_json::from_str::<AvatarStyle>(&saved) {
            send_message(ClientRequestMessage::SetAvatarStyle { avatar_style });
        }
    }
    restore_rooms();
}

/// Recreate (or join, if it still exists) every room saved in localStorage.
fn restore_rooms() {
    for room in room_storage::saved_rooms() {
        ws_create_room(room.room_name, room.hashed_password, true, room.room_type);
    }
}

/// Announce the selected user via speech synthesis.
fn speak_selected_user(user_store: &UserStore, selected_user_id: &str) {
    let user_name = user_store
        .user_untracked(selected_user_id)
        .map(|u| u.name.clone())
        .unwrap_or_else(|| "someone".to_string());
    let text = format!("It is {}", user_name);

    if let Some(window) = web_sys::window() {
        if let Ok(synthesis) = window.speech_synthesis() {
            if let Ok(utterance) = web_sys::SpeechSynthesisUtterance::new_with_text(&text) {
                utterance.set_lang("en-US");
                synthesis.speak(&utterance);
            }
        }
    }
}

/// Route an incoming server message to the appropriate stores. Messages for a specific room
/// type are handed to that type's handler, through `rooms`.
fn process_message(
    msg: ClientResponseMessage,
    room_store: RoomStore,
    user_store: UserStore,
    rooms: RoomStores,
    last_error: RwSignal<Option<String>>,
) {
    match msg {
        ClientResponseMessage::OwnData { user } => {
            user_store.own_data(user);
        }
        ClientResponseMessage::RoomJoined {
            room_name,
            hashed_password,
            room_type,
            users,
        } => {
            let user_ids: Vec<String> = users.iter().map(|u| u.user_id.clone()).collect();
            user_store.room_joined(&users);
            rooms.room_joined(room_type, &room_name, &user_ids);
            room_storage::save_room(&room_name, &hashed_password, room_type);
            room_store.room_joined(room_name, hashed_password, room_type, users);
        }
        ClientResponseMessage::UserJoined { room_name, user } => {
            user_store.user_updated(user.clone());
            if let Some(room_type) = room_store.room_type_untracked(&room_name) {
                rooms.user_joined(room_type, &room_name, &user.user_id);
            }
            room_store.user_joined(&room_name, user);
        }
        ClientResponseMessage::UserLeft { room_name, user_id } => {
            if let Some(room_type) = room_store.room_type_untracked(&room_name) {
                rooms.user_left(room_type, &room_name, &user_id);
            }
            room_store.user_left(&room_name, &user_id);
        }
        ClientResponseMessage::UserUpdated { user } => {
            room_store.user_updated(&user);
            user_store.user_updated(user);
        }
        ClientResponseMessage::RoomMessage { room_name, payload } => {
            match room_store.room_type_untracked(&room_name) {
                Some(room_type) => {
                    let ctx = HandlerCtx {
                        room_store,
                        user_store,
                        last_error,
                    };
                    rooms.handle_message(room_type, &room_name, &payload, ctx);
                }
                None => log::warn!("Room message for unknown room '{}'", room_name),
            }
        }
        ClientResponseMessage::Randomized {
            room_name,
            selected_user_id,
        } => {
            room_store.randomized(&room_name, &selected_user_id);
            speak_selected_user(&user_store, &selected_user_id);
        }
        // Surface server errors to the user
        ClientResponseMessage::InvalidRoomName => {
            last_error.set(Some(
                "Invalid room name. Use 1-50 characters: letters, digits, hyphens, underscores."
                    .to_string(),
            ));
        }
        ClientResponseMessage::WrongPassword { room_name } => {
            last_error.set(Some(format!("Wrong password for room '{}'.", room_name)));
        }
        ClientResponseMessage::AlreadyInRoom { room_name } => {
            last_error.set(Some(format!("You are already in room '{}'.", room_name)));
        }
        ClientResponseMessage::Error { msg } => {
            last_error.set(Some(msg));
        }
    }
}

// Public API functions

/// Register with the server. Guards against duplicate registration.
pub fn ws_register() {
    let ws_ctx = expect_context::<WsContext>();
    if ws_ctx.registered.get_untracked() {
        return;
    }
    ws_ctx.registered.set(true);
    send_message(ClientRequestMessage::Register);
}

pub fn ws_set_name(name: String) {
    if let Some(storage) = room_storage::local_storage() {
        let _ = storage.set_item("name", &name);
    }
    send_message(ClientRequestMessage::SetName { name });
}

pub fn ws_set_avatar_style(avatar_style: AvatarStyle) {
    if let Some(storage) = room_storage::local_storage() {
        if let Ok(json) = serde_json::to_string(&avatar_style) {
            let _ = storage.set_item("avatar_style", &json);
        }
    }
    send_message(ClientRequestMessage::SetAvatarStyle { avatar_style });
}

pub fn ws_join_room(room_name: String, password: String, password_is_hash: bool) {
    send_message(ClientRequestMessage::JoinRoom {
        room_name,
        password,
        password_is_hash,
    });
}

pub fn ws_create_room(
    room_name: String,
    password: String,
    password_is_hash: bool,
    room_type: RoomType,
) {
    send_message(ClientRequestMessage::CreateRoom {
        room_name,
        password,
        password_is_hash,
        room_type,
    });
}

pub fn ws_leave_room(room_name: String) {
    room_storage::forget_room(&room_name);
    send_message(ClientRequestMessage::LeaveRoom { room_name });
}

pub fn ws_room_msg(room_name: String, payload: String) {
    send_message(ClientRequestMessage::RoomMessage { room_name, payload })
}

#[cfg(test)]
mod ws_test;
