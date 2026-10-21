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

use crate::components::avatar::Avatar;
use icons::Eye;
use leptos::prelude::*;
use sizematters_shared::UserData;

/// Side panel listing a room's spectators. Hidden when there are none.
#[component]
pub fn SpectatorList(#[prop(into)] spectators: Signal<Vec<UserData>>) -> impl IntoView {
    view! {
        <Show when=move || spectators.with(|s| !s.is_empty())>
            <div class="flex w-36 shrink-0 flex-col gap-2 rounded-lg border p-3">
                <h3 class="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                    <Eye class="size-3.5" />
                    "Spectators"
                </h3>
                <div class="flex flex-col gap-2">
                    <For
                        each=move || spectators.get()
                        key=|user| user.user_id.clone()
                        let:user
                    >
                        {
                            // <For> keeps a row as long as its key is unchanged, so read the
                            // user again to pick up renames and avatar changes.
                            let uid = user.user_id.clone();
                            let current = Memo::new(move |_| {
                                spectators.with(|s| s.iter().find(|u| u.user_id == uid).cloned())
                            });
                            view! {
                                <div class="flex items-center gap-2">
                                    <div class="size-6 shrink-0 overflow-hidden rounded-full">
                                        {move || view! { <Avatar avatar=current.get().map(|u| u.avatar).unwrap_or_default() /> }}
                                    </div>
                                    <span class="truncate text-sm">{move || current.get().map(|u| u.name).unwrap_or_default()}</span>
                                </div>
                            }
                        }
                    </For>
                </div>
            </div>
        </Show>
    }
}
