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

use crate::components::gravatar::Gravatar;
use crate::stores::{UserStore, VoteStore};
use leptos::prelude::*;

#[component]
pub fn UserCard(user_id: String, room_name: String) -> impl IntoView {
    let user_store = expect_context::<UserStore>();
    let vote_store = expect_context::<VoteStore>();

    let uid = user_id.clone();
    let user_data = Memo::new(move |_| {
        let users = user_store.user_signal().get();
        users.get(&uid).cloned()
    });

    let uid2 = user_id.clone();
    let rn2 = room_name.clone();
    let user_vote = Memo::new(move |_| {
        let votes = vote_store.votes_signal().get();
        votes.get(&rn2).and_then(|rv| rv.get(&uid2)).cloned()
    });

    let name = move || user_data.get().map(|u| u.name.clone()).unwrap_or_default();

    // Memo for gravatar_id so the <img> only re-creates when it actually changes.
    let gravatar_id = Memo::new(move |_| {
        user_data
            .get()
            .map(|u| u.gravatar_id.clone())
            .unwrap_or_default()
    });

    let has_voted = move || user_vote.get().map(|v| v.has_voted).unwrap_or(false);

    let vote_value = move || {
        user_vote
            .get()
            .and_then(|v| v.value)
            .map(|v| v.to_string())
            .unwrap_or_default()
    };

    let order = move || calculate_order(&name());

    let card_class = move || {
        let mut cls = "card user-card".to_string();
        if has_voted() {
            cls.push_str(" has-voted");
        }
        cls
    };

    view! {
        <div class=card_class style:order=order>
            <div class="card-header">
                <div class="card-title">
                    <span class="name">{name}</span>
                    <span class="vote-badge" aria-hidden="true">
                        <span class="material-icons voted">"check"</span>
                        <span class="material-icons not-voted">"help"</span>
                    </span>
                </div>
            </div>
            <div class="card-media">
                {move || {
                    let gid = gravatar_id.get();
                    view! { <Gravatar gravatar_id=gid /> }
                }}
                <div class="user-size">{vote_value}</div>
            </div>
        </div>
    }
}

/// Compute a CSS `order` value for alphabetical sorting of user cards.
/// Uses Unicode scalar values, clamped to avoid overflow.
fn calculate_order(name: &str) -> String {
    let lower = name.to_lowercase();
    let chars: Vec<char> = lower.chars().take(4).collect();
    let padded: Vec<u32> = (0..4)
        .map(|i| {
            if i < chars.len() {
                // Clamp to 2 digits: 'a'=10, 'z'=35, non-alpha gets 10
                let c = chars[i] as u32;
                let a = 'a' as u32;
                if c >= a && c <= a + 25 {
                    c - a + 10
                } else {
                    10
                }
            } else {
                10
            }
        })
        .collect();
    padded
        .iter()
        .map(|n| format!("{:02}", n))
        .collect::<String>()
}
