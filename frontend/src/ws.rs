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

use sizematters_shared::{ClientRequestMessage, ClientResponseMessage};
use crate::stores::{RoomStore, UserStore, VoteStore};
use leptos::prelude::*;
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
    vote_store: VoteStore,
}

impl WsContext {
    pub fn new(room_store: RoomStore, user_store: UserStore, vote_store: VoteStore) -> Self {
        Self {
            connected: RwSignal::new(false),
            last_error: RwSignal::new(None),
            initiated: RwSignal::new(false),
            registered: RwSignal::new(false),
            room_store,
            user_store,
            vote_store,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected.get()
    }

    pub fn last_error(&self) -> ReadSignal<Option<String>> {
        self.last_error.read_only()
    }

    pub fn clear_error(&self) {
        self.last_error.set(None);
    }

    /// Open a WebSocket connection. Guards against duplicate calls.
    pub fn connect(&self) {
        self.initiated.set(true);

        // Prevent duplicate connections
        let already_connected = SOCKET.with(|s| s.borrow().is_some());
        if already_connected || self.connected.get_untracked() {
            return;
        }

        let location = web_sys::window().unwrap().location();
        let protocol = location.protocol().unwrap();
        let host = location.host().unwrap();
        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
        let url = format!("{}//{}/ws", ws_protocol, host);

        let socket = match WebSocket::new(&url) {
            Ok(s) => s,
            Err(err) => {
                log::error!("Failed to create WebSocket: {:?}", err);
                self.last_error.set(Some("Failed to connect to server.".to_string()));
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
        let vote_store = self.vote_store;

        let socket_clone = socket.clone();

        // onopen
        let onopen = Closure::<dyn Fn()>::new(move || {
            log::info!("WebSocket connected");
            connected.set(true);
            registered.set(false);
            RECONNECT_ATTEMPTS.with(|a| a.set(0));
            SOCKET.with(|s| {
                *s.borrow_mut() = Some(socket_clone.clone());
            });
            restore_data();
        });
        socket.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        // onmessage
        let onmessage = Closure::<dyn Fn(web_sys::MessageEvent)>::new(move |e: web_sys::MessageEvent| {
            if let Some(text) = e.data().as_string() {
                match serde_json::from_str::<ClientResponseMessage>(&text) {
                    Ok(msg) => process_message(msg, room_store, user_store, vote_store, last_error),
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
    static SOCKET: std::cell::RefCell<Option<WebSocket>> = std::cell::RefCell::new(None);
    static RECONNECT_ATTEMPTS: std::cell::Cell<u32> = std::cell::Cell::new(0);
}

const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Schedule an automatic reconnection with exponential backoff.
fn schedule_reconnect(ctx: WsContext) {
    if !ctx.initiated.get_untracked() {
        return;
    }

    let attempts = RECONNECT_ATTEMPTS.with(|a| {
        let v = a.get();
        a.set(v + 1);
        v
    });

    if attempts >= MAX_RECONNECT_ATTEMPTS {
        log::warn!("Max reconnection attempts reached");
        ctx.last_error.set(Some("Connection lost. Please refresh the page.".to_string()));
        return;
    }

    // Exponential backoff: 1s, 2s, 4s, 8s, 16s
    let delay = 1000 * 2u32.pow(attempts);
    log::info!("Reconnecting in {}ms (attempt {})", delay, attempts + 1);

    set_timeout(
        move || {
            if !ctx.connected.get_untracked() {
                ctx.connect();
            }
        },
        std::time::Duration::from_millis(delay as u64),
    );
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

/// Restore user name and avatar from localStorage after reconnecting.
fn restore_data() {
    if let Some(name) = crate::storage::load_name() {
        send_message(ClientRequestMessage::SetName { name });
    }
    if let Some(avatar) = crate::storage::load_avatar() {
        send_message(ClientRequestMessage::SetAvatar { avatar });
    }
}

/// Announce the selected user via speech synthesis.
fn speak_selected_user(user_store: &UserStore, selected_user_id: &str) {
    let user_name = user_store.user_untracked(selected_user_id)
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

/// Route an incoming server message to the appropriate stores.
fn process_message(
    msg: ClientResponseMessage,
    room_store: RoomStore,
    user_store: UserStore,
    vote_store: VoteStore,
    last_error: RwSignal<Option<String>>,
) {
    match msg {
        ClientResponseMessage::OwnData { user } => {
            user_store.own_data(user);
        }
        ClientResponseMessage::RoomJoined {
            room_name,
            hashed_password,
            users,
            votes_cast,
        } => {
            crate::storage::save_recent_room(&room_name, &hashed_password);
            let user_ids: Vec<String> = users.iter().map(|u| u.user_id.clone()).collect();
            user_store.room_joined(&users);
            vote_store.room_joined(&room_name, &user_ids, votes_cast);
            room_store.room_joined(room_name, hashed_password, users, votes_cast);
        }
        ClientResponseMessage::UserJoined { room_name, user } => {
            user_store.user_updated(user.clone());
            vote_store.user_joined(&room_name, &user.user_id);
            room_store.user_joined(&room_name, user);
        }
        ClientResponseMessage::UserLeft { room_name, user_id } => {
            vote_store.user_left(&room_name, &user_id);
            room_store.user_left(&room_name, &user_id);
        }
        ClientResponseMessage::UserUpdated { user } => {
            user_store.user_updated(user);
        }
        ClientResponseMessage::OwnVote { room_name, size } => {
            let own_id = user_store.own_user_id_untracked();
            vote_store.own_vote(&room_name, &own_id, size);
        }
        ClientResponseMessage::VoteStatus { room_name, votes } => {
            room_store.vote_status(&room_name, &votes);
            vote_store.vote_status(&room_name, &votes);
        }
        ClientResponseMessage::VoteResults { room_name, votes } => {
            vote_store.vote_results(&room_name, &votes);
        }
        ClientResponseMessage::NewVote { room_name } => {
            room_store.new_vote(&room_name);
            vote_store.new_vote(&room_name);
        }
        ClientResponseMessage::Randomized { room_name, selected_user_id } => {
            room_store.randomized(&room_name, &selected_user_id);
            speak_selected_user(&user_store, &selected_user_id);
        }
        // Surface server errors to the user
        ClientResponseMessage::InvalidRoomName => {
            last_error.set(Some("Invalid room name. Use 1-50 characters: letters, digits, hyphens, underscores.".to_string()));
        }
        ClientResponseMessage::WrongPassword { room_name } => {
            last_error.set(Some(format!("Wrong password for room '{}'.", room_name)));
        }
        ClientResponseMessage::AlreadyInRoom { room_name } => {
            last_error.set(Some(format!("You are already in room '{}'.", room_name)));
        }
        ClientResponseMessage::CannotJoinMultipleRooms => {
            last_error.set(Some("You can only be in one room at a time.".to_string()));
        }
        ClientResponseMessage::VotingOver => {
            last_error.set(Some("Voting is already over.".to_string()));
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
    crate::storage::save_name(&name);
    send_message(ClientRequestMessage::SetName { name });
}

pub fn ws_set_avatar(email: String) {
    crate::storage::save_avatar(&email);
    send_message(ClientRequestMessage::SetAvatar { avatar: email });
}

pub fn ws_join_room(room_name: String, password: String, password_is_hash: bool) {
    send_message(ClientRequestMessage::JoinRoom {
        room_name,
        password,
        password_is_hash,
    });
}

pub fn ws_leave_room(room_name: String) {
    send_message(ClientRequestMessage::LeaveRoom { room_name });
}

pub fn ws_vote(room_name: String, size: u64) {
    send_message(ClientRequestMessage::Vote { room_name, size });
}

pub fn ws_new_vote(room_name: String) {
    send_message(ClientRequestMessage::NewVote { room_name });
}

pub fn ws_randomize(room_name: String) {
    send_message(ClientRequestMessage::Randomize { room_name });
}
