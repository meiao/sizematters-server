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
use crate::rooms::HandlerCtx;
use leptos::prelude::*;
use sizematters_shared::messages::sizing::SizingResponseMessage;

/// Handle the payload of a `RoomMessage` sent to a sizing room.
pub fn handle_message(payload: &str, room_name: &str, vote_store: VoteStore, ctx: HandlerCtx) {
    let msg = match serde_json::from_str::<SizingResponseMessage>(payload) {
        Ok(msg) => msg,
        Err(err) => {
            log::warn!("Failed to parse sizing room message: {}", err);
            return;
        }
    };
    match msg {
        SizingResponseMessage::OwnVote { size } => {
            let own_id = ctx.user_store.own_user_id_untracked();
            vote_store.own_vote(room_name, &own_id, size);
        }
        SizingResponseMessage::VoteStatus {
            votes,
            voting_system,
            spectators,
        } => {
            vote_store.set_voting_system(room_name, voting_system);
            vote_store.vote_status(room_name, &votes);
            vote_store.set_spectators(room_name, &spectators);
        }
        SizingResponseMessage::VoteResults { votes } => {
            vote_store.mark_as_voters(room_name, votes.keys().cloned());
            vote_store.vote_results(room_name, &votes);
        }
        // The server drops all votes when the system changes, without a NewVote.
        SizingResponseMessage::VotingSystemChanged { new_system } => {
            vote_store.set_voting_system(room_name, new_system);
            ctx.room_store.clear_selected_user(room_name);
            vote_store.new_vote(room_name);
        }
        SizingResponseMessage::NewVote => {
            ctx.room_store.clear_selected_user(room_name);
            vote_store.new_vote(room_name);
        }
        SizingResponseMessage::VotingOver => {
            ctx.last_error
                .set(Some("Voting is already over.".to_string()));
        }
    }
}
