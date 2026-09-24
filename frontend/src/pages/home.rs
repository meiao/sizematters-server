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

use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Card class="max-w-xl">
            <CardHeader>
                <CardTitle class="text-2xl">"Welcome to Size Matters."</CardTitle>
            </CardHeader>
            <CardContent class="flex flex-col gap-2 text-sm leading-relaxed">
                <p>"Here you can get a room with all your developer team and see who has the bigger size."</p>
                <p>"Or fight until you reach an agreement."</p>
            </CardContent>
        </Card>
    }
}
