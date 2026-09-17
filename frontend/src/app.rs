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

use crate::components::main_menu::MainMenu;
use crate::components::no_menu::NoMenu;
use crate::pages::connecting::ConnectingPage;
use crate::pages::error::ErrorPage;
use crate::pages::home::HomePage;
use crate::pages::main_view::MainPage;
use crate::stores::{RoomStore, UserStore, VoteStore};
use crate::ws::WsContext;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::use_navigate;
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    let user_store = UserStore::new();
    let room_store = RoomStore::new();
    let vote_store = VoteStore::new();
    let ws_ctx = WsContext::new(room_store, user_store, vote_store);

    provide_context(user_store);
    provide_context(room_store);
    provide_context(vote_store);
    provide_context(ws_ctx);

    view! {
        <Router>
            <div id="app">
                <header class="toolbar">
                    <h1>"Size Matters"</h1>
                </header>
                <div class="app-body">
                    <nav id="nav" aria-label="Sidebar">
                        <MenuRouter />
                    </nav>
                    <main class="app-content">
                        <Routes fallback=|| "Not found">
                            <Route path=path!("/") view=HomePage />
                            <Route path=path!("/main") view=MainPage />
                            <Route path=path!("/room/:room_name/:password") view=ConnectingPage />
                            <Route path=path!("/error/:error_type") view=ErrorPage />
                        </Routes>
                    </main>
                </div>
            </div>
        </Router>
    }
}

#[component]
fn MenuRouter() -> impl IntoView {
    let ws_ctx = expect_context::<WsContext>();
    let navigate = use_navigate();

    let connected = Memo::new(move |_| ws_ctx.is_connected());

    // Navigate to /main once the connection succeeds. This lives here, in a
    // component that stays mounted regardless of connection state, rather than
    // inside `NoMenu` — `NoMenu` gets unmounted by the `<Show>` below the
    // instant `connected` flips true, which raced with (and could silently
    // drop) an effect inside `NoMenu` trying to react to that same flip.
    let nav = navigate.clone();
    Effect::new(move |_| {
        if ws_ctx.is_initiated() && connected.get() {
            nav("/main", Default::default());
        }
    });

    // Timeout fallback: if a connection attempt doesn't succeed within 5s,
    // send the user to the error page.
    let nav2 = navigate.clone();
    Effect::new(move |prev: Option<bool>| {
        let initiated = ws_ctx.is_initiated();
        if initiated && prev != Some(true) {
            let nav = nav2.clone();
            set_timeout(
                move || {
                    if !ws_ctx.is_connected() {
                        nav("/error/connection", Default::default());
                    }
                },
                std::time::Duration::from_secs(5),
            );
        }
        initiated
    });

    view! {
        <Show
            when=move || connected.get()
            fallback=|| view! { <NoMenu /> }
        >
            <MainMenu />
        </Show>
    }
}
