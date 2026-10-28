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

use super::*;
use crate::actors::rooms::test_support::*;
use sizematters_shared::RoomType;
use std::collections::HashSet;

const REGISTER: &str = r#"{"type":"RegisterToVote"}"#;
const NEW_VOTE: &str = r#"{"type":"NewVote"}"#;
const RANDOMIZE: &str = r#"{"type":"Randomize"}"#;

fn vote(size: &str) -> String {
    format!(r#"{{"type":"Vote","data":{{"size":"{size}"}}}}"#)
}

fn change_system(system: &str) -> String {
    format!(r#"{{"type":"ChangeVotingSystem","data":{{"new_system":"{system}"}}}}"#)
}

fn revoke(target: &str) -> String {
    format!(r#"{{"type":"RevokeVotingRights","data":{{"target_id":"{target}"}}}}"#)
}

type Fixture = RoomFixture<SizingRequestMessage>;

fn new_room() -> Fixture {
    Fixture::new(RoomType::Sizing, Box::new(SizingBehavior::new()))
}

/// A room where every given user joined and registered to vote.
async fn with_voters(user_ids: &[&str]) -> Fixture {
    let mut fixture = new_room();
    for user_id in user_ids {
        fixture.join(user_id).await;
        fixture.send(user_id, REGISTER).await;
    }
    fixture.drain_all().await;
    fixture
}

fn last_status(
    msgs: &[SizingResponseMessage],
) -> (HashMap<String, bool>, VotingSystem, Vec<String>) {
    msgs.iter()
        .rev()
        .find_map(|m| match m {
            SizingResponseMessage::VoteStatus {
                votes,
                voting_system,
                spectators,
            } => Some((votes.clone(), *voting_system, spectators.clone())),
            _ => None,
        })
        .expect("expected a VoteStatus")
}

fn results(msgs: &[SizingResponseMessage]) -> Option<HashMap<String, String>> {
    msgs.iter().rev().find_map(|m| match m {
        SizingResponseMessage::VoteResults { votes } => Some(votes.clone()),
        _ => None,
    })
}

fn strings(map: &[(&str, &str)]) -> HashMap<String, String> {
    map.iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn flags(map: &[(&str, bool)]) -> HashMap<String, bool> {
    map.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

#[actix_rt::test]
async fn joining_user_gets_the_vote_status_as_a_spectator() {
    let mut fixture = with_voters(&["u1"]).await;

    fixture.join("u2").await;

    let msgs = fixture.payloads::<SizingResponseMessage>("u2").await;
    assert_eq!(msgs.len(), 1);
    let (votes, system, spectators) = last_status(&msgs);
    assert_eq!(votes, flags(&[("u1", false)]));
    assert_eq!(system, VotingSystem::Fibonacci);
    assert_eq!(spectators, vec!["u2"]);
}

#[actix_rt::test]
async fn registering_to_vote_is_broadcast() {
    let mut fixture = with_voters(&["u1"]).await;
    fixture.join("u2").await;
    fixture.drain_all().await;

    fixture.send("u2", REGISTER).await;

    for user_id in ["u1", "u2"] {
        let (votes, _, spectators) =
            last_status(&fixture.payloads::<SizingResponseMessage>(user_id).await);
        assert_eq!(votes, flags(&[("u1", false), ("u2", false)]));
        assert!(spectators.is_empty());
    }
}

#[actix_rt::test]
async fn spectators_cannot_vote() {
    let mut fixture = new_room();
    fixture.join("u1").await;
    fixture.drain_all().await;

    fixture.send("u1", &vote("3")).await;

    assert_eq!(
        errors(&fixture.take("u1").await),
        vec!["You must register to vote to be able to vote"]
    );
}

#[actix_rt::test]
async fn vote_is_confirmed_to_the_voter_and_hidden_from_others() {
    let fixture = with_voters(&["u1", "u2"]).await;

    fixture.send("u1", &vote("5")).await;

    let own = fixture.payloads::<SizingResponseMessage>("u1").await;
    assert!(matches!(&own[0], SizingResponseMessage::OwnVote { size } if size == "5"));
    assert_eq!(last_status(&own).0, flags(&[("u1", true), ("u2", false)]));
    assert!(results(&own).is_none());

    let other = fixture.payloads::<SizingResponseMessage>("u2").await;
    assert_eq!(other.len(), 1, "only the status, not the value");
    assert_eq!(last_status(&other).0, flags(&[("u1", true), ("u2", false)]));
}

#[actix_rt::test]
async fn results_are_revealed_once_everyone_voted() {
    let fixture = with_voters(&["u1", "u2"]).await;

    fixture.send("u1", &vote("5")).await;
    fixture.send("u2", &vote("8")).await;

    let expected = strings(&[("u1", "5"), ("u2", "8")]);
    assert_eq!(
        results(&fixture.payloads::<SizingResponseMessage>("u1").await),
        Some(expected.clone())
    );
    assert_eq!(
        results(&fixture.payloads::<SizingResponseMessage>("u2").await),
        Some(expected)
    );
}

#[actix_rt::test]
async fn spectators_are_not_waited_for_and_see_the_results() {
    let mut fixture = with_voters(&["u1"]).await;
    fixture.join("watcher").await;
    fixture.drain_all().await;

    fixture.send("u1", &vote("2")).await;

    assert_eq!(
        results(&fixture.payloads::<SizingResponseMessage>("watcher").await),
        Some(strings(&[("u1", "2")]))
    );
}

#[actix_rt::test]
async fn changing_a_vote_replaces_it_without_a_new_status() {
    let fixture = with_voters(&["u1", "u2"]).await;
    fixture.send("u1", &vote("5")).await;
    fixture.drain_all().await;

    fixture.send("u1", &vote("13")).await;

    let own = fixture.payloads::<SizingResponseMessage>("u1").await;
    assert_eq!(own.len(), 1);
    assert!(matches!(&own[0], SizingResponseMessage::OwnVote { size } if size == "13"));
    assert!(fixture.take("u2").await.is_empty());

    fixture.send("u2", &vote("1")).await;
    assert_eq!(
        results(&fixture.payloads::<SizingResponseMessage>("u2").await),
        Some(strings(&[("u1", "13"), ("u2", "1")]))
    );
}

#[actix_rt::test]
async fn votes_outside_the_voting_system_are_rejected() {
    let fixture = with_voters(&["u1", "u2"]).await;

    for invalid in ["4", "M", "", "100"] {
        fixture.send("u1", &vote(invalid)).await;
        assert_eq!(
            errors(&fixture.take("u1").await),
            vec!["Invalid vote value"],
            "{invalid:?}"
        );
    }
    assert!(fixture.take("u2").await.is_empty());
}

#[actix_rt::test]
async fn voting_after_the_results_is_refused() {
    let fixture = with_voters(&["u1"]).await;
    fixture.send("u1", &vote("3")).await;
    fixture.drain_all().await;

    fixture.send("u1", &vote("5")).await;

    let msgs = fixture.payloads::<SizingResponseMessage>("u1").await;
    assert_eq!(msgs.len(), 1);
    assert!(matches!(msgs[0], SizingResponseMessage::VotingOver));
}

#[actix_rt::test]
async fn new_vote_resets_the_round() {
    let fixture = with_voters(&["u1", "u2"]).await;
    fixture.send("u1", &vote("3")).await;
    fixture.send("u2", &vote("3")).await;
    fixture.drain_all().await;

    fixture.send("u2", NEW_VOTE).await;

    for user_id in ["u1", "u2"] {
        let msgs = fixture.payloads::<SizingResponseMessage>(user_id).await;
        assert_eq!(msgs.len(), 1);
        assert!(matches!(msgs[0], SizingResponseMessage::NewVote));
    }

    // Previous votes are gone: one vote is no longer enough for results.
    fixture.send("u1", &vote("8")).await;
    let msgs = fixture.payloads::<SizingResponseMessage>("u1").await;
    assert!(matches!(&msgs[0], SizingResponseMessage::OwnVote { size } if size == "8"));
    assert_eq!(last_status(&msgs).0, flags(&[("u1", true), ("u2", false)]));
    assert!(results(&msgs).is_none());
}

#[actix_rt::test]
async fn new_vote_from_outside_the_room_is_ignored() {
    let fixture = with_voters(&["u1"]).await;
    fixture.send("u1", &vote("3")).await;
    fixture.drain_all().await;

    fixture.send("stranger", NEW_VOTE).await;

    assert!(fixture.take("u1").await.is_empty());
}

#[actix_rt::test]
async fn changing_the_voting_system_clears_votes() {
    let fixture = with_voters(&["u1", "u2"]).await;
    fixture.send("u1", &vote("3")).await;
    fixture.drain_all().await;

    fixture.send("u1", &change_system("TShirt")).await;

    for user_id in ["u1", "u2"] {
        let msgs = fixture.payloads::<SizingResponseMessage>(user_id).await;
        assert_eq!(msgs.len(), 1);
        assert!(matches!(
            msgs[0],
            SizingResponseMessage::VotingSystemChanged {
                new_system: VotingSystem::TShirt
            }
        ));
    }

    // Old values are rejected, new ones accepted, and the old vote was dropped.
    fixture.send("u1", &vote("3")).await;
    assert_eq!(
        errors(&fixture.take("u1").await),
        vec!["Invalid vote value"]
    );
    fixture.send("u1", &vote("XL")).await;
    let (votes, system, _) = last_status(&fixture.payloads::<SizingResponseMessage>("u1").await);
    assert_eq!(system, VotingSystem::TShirt);
    assert_eq!(votes, flags(&[("u1", true), ("u2", false)]));
}

#[actix_rt::test]
async fn changing_the_voting_system_reopens_a_finished_round() {
    let fixture = with_voters(&["u1"]).await;
    fixture.send("u1", &vote("3")).await;
    fixture.send("u1", &change_system("FistToFive")).await;
    fixture.drain_all().await;

    fixture.send("u1", &vote("4")).await;

    let msgs = fixture.payloads::<SizingResponseMessage>("u1").await;
    assert_eq!(results(&msgs), Some(strings(&[("u1", "4")])));
}

#[actix_rt::test]
async fn selecting_the_current_voting_system_does_nothing() {
    let fixture = with_voters(&["u1", "u2"]).await;
    fixture.send("u1", &vote("3")).await;
    fixture.drain_all().await;

    fixture.send("u2", &change_system("Fibonacci")).await;
    assert!(fixture.take("u1").await.is_empty());

    // u1's vote survived.
    fixture.send("u2", &vote("5")).await;
    assert_eq!(
        results(&fixture.payloads::<SizingResponseMessage>("u2").await),
        Some(strings(&[("u1", "3"), ("u2", "5")]))
    );
}

#[actix_rt::test]
async fn randomize_picks_someone_in_the_room() {
    let mut fixture = with_voters(&["u1", "u2"]).await;
    fixture.join("watcher").await;
    fixture.drain_all().await;
    let members: HashSet<&str> = HashSet::from(["u1", "u2", "watcher"]);

    let mut picked = HashSet::new();
    for _ in 0..50 {
        fixture.send("u1", RANDOMIZE).await;
        let mut selections = Vec::new();
        for user_id in &members {
            let msgs = fixture.take(user_id).await;
            assert_eq!(msgs.len(), 1);
            match &msgs[0] {
                ClientResponseMessage::Randomized {
                    room_name,
                    selected_user_id,
                } => {
                    assert_eq!(room_name, ROOM);
                    selections.push(selected_user_id.clone());
                }
                other => panic!("expected Randomized, got {other:?}"),
            }
        }
        // Everyone is told about the same person.
        assert!(selections.windows(2).all(|w| w[0] == w[1]));
        assert!(members.contains(selections[0].as_str()));
        picked.insert(selections[0].clone());
    }
    // Spectators can be picked too. With 50 draws, missing someone is (2/3)^50 unlikely.
    assert_eq!(picked.len(), 3);
}

#[actix_rt::test]
async fn revoking_voting_rights_drops_the_vote() {
    let fixture = with_voters(&["u1", "u2", "u3"]).await;
    fixture.send("u1", &vote("3")).await;
    fixture.send("u2", &vote("5")).await;
    fixture.drain_all().await;

    // u3 hasn't voted, so removing them ends the round.
    fixture.send("u1", &revoke("u3")).await;

    let msgs = fixture.payloads::<SizingResponseMessage>("u3").await;
    let (votes, _, spectators) = last_status(&msgs);
    assert_eq!(votes, flags(&[("u1", true), ("u2", true)]));
    assert_eq!(spectators, vec!["u3"]);
    assert_eq!(results(&msgs), Some(strings(&[("u1", "3"), ("u2", "5")])));

    // And they can no longer vote.
    fixture.send("u1", NEW_VOTE).await;
    fixture.send("u3", &vote("1")).await;
    assert_eq!(
        errors(&fixture.take("u3").await),
        vec!["You must register to vote to be able to vote"]
    );
}

#[actix_rt::test]
async fn revoked_vote_is_not_in_the_results() {
    let fixture = with_voters(&["u1", "u2"]).await;
    fixture.send("u2", &vote("21")).await;
    fixture.send("u1", &revoke("u2")).await;
    fixture.drain_all().await;

    fixture.send("u1", &vote("1")).await;

    assert_eq!(
        results(&fixture.payloads::<SizingResponseMessage>("u1").await),
        Some(strings(&[("u1", "1")]))
    );
}

#[actix_rt::test]
async fn revoking_rights_of_a_spectator_does_nothing() {
    let mut fixture = with_voters(&["u1"]).await;
    fixture.join("u2").await;
    fixture.drain_all().await;

    fixture.send("u1", &revoke("u2")).await;

    assert!(fixture.take("u1").await.is_empty());
    assert!(fixture.take("u2").await.is_empty());
}

#[actix_rt::test]
async fn revoking_rights_of_an_unknown_user_is_an_error() {
    let fixture = with_voters(&["u1", "u2"]).await;

    fixture.send("u1", &revoke("ghost")).await;

    assert_eq!(errors(&fixture.take("u1").await), vec!["User not found."]);
    assert!(fixture.take("u2").await.is_empty());
}

#[actix_rt::test]
async fn last_voter_leaving_reveals_the_results() {
    let fixture = with_voters(&["u1", "u2"]).await;
    fixture.send("u1", &vote("8")).await;
    fixture.drain_all().await;

    fixture.leave("u2").await;

    let msgs = fixture.payloads::<SizingResponseMessage>("u1").await;
    assert_eq!(last_status(&msgs).0, flags(&[("u1", true)]));
    assert_eq!(results(&msgs), Some(strings(&[("u1", "8")])));
}

#[actix_rt::test]
async fn leaving_voter_takes_their_vote_along() {
    let fixture = with_voters(&["u1", "u2", "u3"]).await;
    fixture.send("u1", &vote("8")).await;
    fixture.leave("u1").await;
    fixture.drain_all().await;

    fixture.send("u2", &vote("1")).await;
    fixture.send("u3", &vote("2")).await;

    assert_eq!(
        results(&fixture.payloads::<SizingResponseMessage>("u2").await),
        Some(strings(&[("u2", "1"), ("u3", "2")]))
    );
}

#[actix_rt::test]
async fn joining_after_the_round_ended_shows_the_results() {
    let mut fixture = with_voters(&["u1"]).await;
    fixture.send("u1", &vote("13")).await;

    fixture.join("late").await;

    let msgs = fixture.payloads::<SizingResponseMessage>("late").await;
    assert_eq!(msgs.len(), 2);
    assert!(matches!(msgs[0], SizingResponseMessage::VoteStatus { .. }));
    assert_eq!(results(&msgs), Some(strings(&[("u1", "13")])));
}

#[actix_rt::test]
async fn room_with_only_spectators_never_ends_the_round() {
    let mut fixture = new_room();
    fixture.join("u1").await;
    fixture.join("u2").await;
    fixture.drain_all().await;

    fixture.leave("u2").await;

    let msgs = fixture.payloads::<SizingResponseMessage>("u1").await;
    assert!(results(&msgs).is_none());
}

#[actix_rt::test]
async fn votes_from_outside_the_room_are_ignored() {
    let fixture = with_voters(&["u1"]).await;

    fixture.send("stranger", &vote("3")).await;

    assert!(fixture.take("u1").await.is_empty());
}
