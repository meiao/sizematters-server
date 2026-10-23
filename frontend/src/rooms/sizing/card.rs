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

use super::VoteStore;
use crate::rooms::common::{send_room_msg, PlayerCard};
use crate::stores::UserStore;
use leptos::prelude::*;
use sizematters_shared::messages::sizing::SizingRequestMessage;

/// A voter's card. Shows whether they voted, and flips over to show their vote once the
/// results are revealed.
#[component]
pub fn VoteCard(
    user_id: String,
    room_name: String,
    /// The results of the round are shown.
    #[prop(into)]
    revealed: Signal<bool>,
    /// Picked by the randomizer.
    #[prop(into)]
    selected: Signal<bool>,
) -> impl IntoView {
    let user_store = expect_context::<UserStore>();
    let vote_store = expect_context::<VoteStore>();

    let uid = user_id.clone();
    let user_data = Memo::new(move |_| {
        user_store
            .user_signal()
            .with(|users| users.get(&uid).cloned())
    });

    let uid = user_id.clone();
    let rn = room_name.clone();
    let user_vote = Memo::new(move |_| {
        vote_store
            .votes_signal()
            .with(|votes| votes.get(&rn).and_then(|rv| rv.get(&uid)).cloned())
    });

    let has_voted = Signal::derive(move || user_vote.get().is_some_and(|v| v.has_voted));
    let vote_value = move || user_vote.get().and_then(|v| v.value).unwrap_or_default();

    let revoke = Callback::new(move |_| {
        send_room_msg(
            &room_name,
            &SizingRequestMessage::RevokeVotingRights {
                target_id: user_id.clone(),
            },
        );
    });

    view! {
        <PlayerCard
            name=Signal::derive(move || user_data.get().map(|u| u.name).unwrap_or_default())
            avatar=Signal::derive(move || user_data.get().map(|u| u.avatar).unwrap_or_default())
            done=has_voted
            revealed=Signal::derive(move || revealed.get() && has_voted.get())
            selected=selected
            on_remove=revoke
        >
            <span class="text-4xl font-semibold sm:text-6xl">{vote_value}</span>
        </PlayerCard>
    }
}
