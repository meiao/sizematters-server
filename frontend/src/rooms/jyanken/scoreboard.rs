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

use super::store::JyankenState;
use crate::collate::compare_names;
use crate::stores::UserStore;
use icons::Trophy;
use leptos::prelude::*;

/// Wins per player, best total first. Hidden until the first round ends.
#[component]
pub fn Scoreboard(#[prop(into)] state: Signal<JyankenState>) -> impl IntoView {
    let user_store = expect_context::<UserStore>();

    // (name, wins last round, total wins). Names come from the user store so players who
    // already left still show up under their last known name.
    let rows = Memo::new(move |_| -> Vec<(String, u32, u32)> {
        let known = user_store.user_signal().get();
        let mut rows: Vec<(String, u32, u32)> = state.with(|s| {
            s.scoreboard
                .iter()
                .map(|(id, (last, total))| {
                    let name = known
                        .get(id)
                        .map(|u| u.name.clone())
                        .unwrap_or_else(|| id.clone());
                    (name, *last, *total)
                })
                .collect()
        });
        rows.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| compare_names(&a.0, &b.0)));
        rows
    });

    view! {
        <Show when=move || rows.with(|r| !r.is_empty())>
            <section class="w-64 shrink-0 rounded-lg border p-3" aria-label="Scoreboard">
                <h3 class="mb-2 flex items-center gap-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wide">
                    <Trophy class="size-3.5" />
                    "Scoreboard"
                </h3>
                <table class="w-full text-sm">
                    <thead class="text-left text-xs text-muted-foreground">
                        <tr>
                            <th class="font-medium">"Player"</th>
                            <th class="text-right font-medium">"Last round"</th>
                            <th class="text-right font-medium">"Total"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || rows.get()
                            key=|row| row.clone()
                            let:row
                        >
                            <tr>
                                <td class="truncate py-0.5">{row.0}</td>
                                <td class="py-0.5 text-right tabular-nums">{row.1}</td>
                                <td class="py-0.5 text-right font-semibold tabular-nums">{row.2}</td>
                            </tr>
                        </For>
                    </tbody>
                </table>
            </section>
        </Show>
    }
}
