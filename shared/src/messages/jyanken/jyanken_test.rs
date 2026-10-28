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

#[test]
fn weapon_wire_format() {
    // Weapons use the same adjacently tagged format as messages.
    assert_eq!(
        serde_json::to_value(JyankenWeapon::Scissor).unwrap(),
        json!({"type": "Scissor"})
    );
    assert_eq!(
        serde_json::from_str::<JyankenWeapon>(r#"{"type":"Rock"}"#).unwrap(),
        JyankenWeapon::Rock
    );
}

#[test]
fn shoot_wire_format() {
    assert_eq!(
        serde_json::to_value(JyankenRequestMessage::Shoot {
            weapon: JyankenWeapon::Paper
        })
        .unwrap(),
        json!({"type": "Shoot", "data": {"weapon": {"type": "Paper"}}})
    );
}

#[test]
fn parses_every_request() {
    let payloads = [
        r#"{"type":"JoinGame"}"#,
        r#"{"type":"Shoot","data":{"weapon":{"type":"Rock"}}}"#,
        r#"{"type":"NewGame"}"#,
        r#"{"type":"RemoveFromGame","data":{"target_id":"u2"}}"#,
    ];
    for payload in payloads {
        assert!(
            serde_json::from_str::<JyankenRequestMessage>(payload).is_ok(),
            "{payload}"
        );
    }
}

#[test]
fn rejects_unknown_weapon() {
    assert!(serde_json::from_str::<JyankenRequestMessage>(
        r#"{"type":"Shoot","data":{"weapon":{"type":"Lizard"}}}"#
    )
    .is_err());
}

#[test]
fn scoreboard_entries_serialize_as_pairs() {
    let msg = JyankenResponseMessage::GameResults {
        weapons: HashMap::from([("u1".to_string(), JyankenWeapon::Rock)]),
        scoreboard: HashMap::from([("u1".to_string(), (1, 4))]),
    };
    assert_eq!(
        serde_json::to_value(msg).unwrap(),
        json!({
            "type": "GameResults",
            "data": {
                "weapons": {"u1": {"type": "Rock"}},
                "scoreboard": {"u1": [1, 4]}
            }
        })
    );
}

#[test]
fn every_response_round_trips_through_json() {
    let messages = vec![
        JyankenResponseMessage::NewGame,
        JyankenResponseMessage::OwnShoot {
            weapon: JyankenWeapon::Scissor,
        },
        JyankenResponseMessage::GameStatus {
            selected: HashMap::from([("u1".to_string(), true)]),
            spectators: vec!["u2".to_string()],
        },
        JyankenResponseMessage::GameResults {
            weapons: HashMap::from([("u1".to_string(), JyankenWeapon::Paper)]),
            scoreboard: HashMap::from([("u1".to_string(), (0, 2))]),
        },
        JyankenResponseMessage::GameOver,
    ];
    for msg in messages {
        let json = serde_json::to_string(&msg).unwrap();
        let back: JyankenResponseMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), json);
    }
}
