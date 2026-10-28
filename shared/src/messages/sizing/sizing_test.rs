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
use serde_json::json;
use std::collections::HashSet;
use strum::IntoEnumIterator;

#[test]
fn default_voting_system_is_fibonacci() {
    assert_eq!(VotingSystem::default(), VotingSystem::Fibonacci);
}

#[test]
fn values_per_voting_system() {
    assert_eq!(
        VotingSystem::Fibonacci.values(),
        &["0", "1", "2", "3", "5", "8", "13", "21"]
    );
    assert_eq!(
        VotingSystem::FistToFive.values(),
        &["0", "1", "2", "3", "4", "5"]
    );
    assert_eq!(VotingSystem::TShirt.values(), &["XS", "S", "M", "L", "XL"]);
}

#[test]
fn values_have_no_duplicates() {
    for system in VotingSystem::iter() {
        let unique: HashSet<_> = system.values().iter().collect();
        assert_eq!(unique.len(), system.values().len(), "{system:?}");
    }
}

#[test]
fn display_names_are_unique_and_non_empty() {
    let names: HashSet<&str> = VotingSystem::iter().map(|s| s.display_name()).collect();
    assert_eq!(names.len(), VotingSystem::iter().count());
    assert!(names.iter().all(|n| !n.is_empty()));
}

#[test]
fn request_wire_format() {
    assert_eq!(
        serde_json::to_value(SizingRequestMessage::Vote {
            size: "5".to_string()
        })
        .unwrap(),
        json!({"type": "Vote", "data": {"size": "5"}})
    );
    assert_eq!(
        serde_json::to_value(SizingRequestMessage::ChangeVotingSystem {
            new_system: VotingSystem::TShirt
        })
        .unwrap(),
        json!({"type": "ChangeVotingSystem", "data": {"new_system": "TShirt"}})
    );
    assert_eq!(
        serde_json::to_value(SizingRequestMessage::RegisterToVote).unwrap(),
        json!({"type": "RegisterToVote"})
    );
}

#[test]
fn parses_every_request() {
    let payloads = [
        r#"{"type":"Vote","data":{"size":"8"}}"#,
        r#"{"type":"ChangeVotingSystem","data":{"new_system":"FistToFive"}}"#,
        r#"{"type":"NewVote"}"#,
        r#"{"type":"Randomize"}"#,
        r#"{"type":"RegisterToVote"}"#,
        r#"{"type":"RevokeVotingRights","data":{"target_id":"u2"}}"#,
    ];
    for payload in payloads {
        assert!(
            serde_json::from_str::<SizingRequestMessage>(payload).is_ok(),
            "{payload}"
        );
    }
}

#[test]
fn rejects_unknown_voting_system() {
    assert!(serde_json::from_str::<SizingRequestMessage>(
        r#"{"type":"ChangeVotingSystem","data":{"new_system":"Powers"}}"#
    )
    .is_err());
}

#[test]
fn vote_status_wire_format() {
    let msg = SizingResponseMessage::VoteStatus {
        votes: HashMap::from([("u1".to_string(), true)]),
        voting_system: VotingSystem::Fibonacci,
        spectators: vec!["u2".to_string()],
    };
    assert_eq!(
        serde_json::to_value(msg).unwrap(),
        json!({
            "type": "VoteStatus",
            "data": {
                "votes": {"u1": true},
                "voting_system": "Fibonacci",
                "spectators": ["u2"]
            }
        })
    );
}

#[test]
fn every_response_round_trips_through_json() {
    let messages = vec![
        SizingResponseMessage::OwnVote {
            size: "3".to_string(),
        },
        SizingResponseMessage::VoteStatus {
            votes: HashMap::from([("u1".to_string(), false)]),
            voting_system: VotingSystem::TShirt,
            spectators: vec![],
        },
        SizingResponseMessage::VoteResults {
            votes: HashMap::from([("u1".to_string(), "M".to_string())]),
        },
        SizingResponseMessage::VotingSystemChanged {
            new_system: VotingSystem::FistToFive,
        },
        SizingResponseMessage::NewVote,
        SizingResponseMessage::VotingOver,
    ];
    for msg in messages {
        let json = serde_json::to_string(&msg).unwrap();
        let back: SizingResponseMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}
