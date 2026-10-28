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
use JyankenWeapon::{Paper, Rock, Scissor};

const JOIN_GAME: &str = r#"{"type":"JoinGame"}"#;
const NEW_GAME: &str = r#"{"type":"NewGame"}"#;

fn shoot(weapon: JyankenWeapon) -> String {
    serde_json::to_string(&JyankenRequestMessage::Shoot { weapon }).unwrap()
}

fn remove(target: &str) -> String {
    format!(r#"{{"type":"RemoveFromGame","data":{{"target_id":"{target}"}}}}"#)
}

type Fixture = RoomFixture<JyankenRequestMessage>;

fn new_room() -> Fixture {
    Fixture::new(RoomType::Jyanken, Box::new(JyankenBehavior::new()))
}

/// A room where every given user joined and joined the game.
async fn with_players(user_ids: &[&str]) -> Fixture {
    let mut fixture = new_room();
    for user_id in user_ids {
        fixture.join(user_id).await;
        fixture.send(user_id, JOIN_GAME).await;
    }
    fixture.drain_all().await;
    fixture
}

async fn jyanken(fixture: &Fixture, user_id: &str) -> Vec<JyankenResponseMessage> {
    fixture.payloads(user_id).await
}

/// Plays one round where each player shoots the given weapon; returns the scoreboard.
async fn play_round(
    fixture: &Fixture,
    shots: &[(&str, JyankenWeapon)],
) -> HashMap<String, (u32, u32)> {
    for (user_id, weapon) in shots {
        fixture.send(user_id, &shoot(weapon.clone())).await;
    }
    let msgs = jyanken(fixture, shots[0].0).await;
    fixture.drain_all().await;
    results(&msgs).expect("round should be over").1
}

fn last_status(msgs: &[JyankenResponseMessage]) -> (HashMap<String, bool>, Vec<String>) {
    msgs.iter()
        .rev()
        .find_map(|m| match m {
            JyankenResponseMessage::GameStatus {
                selected,
                spectators,
            } => Some((selected.clone(), spectators.clone())),
            _ => None,
        })
        .expect("expected a GameStatus")
}

type Results = (HashMap<String, JyankenWeapon>, HashMap<String, (u32, u32)>);

fn results(msgs: &[JyankenResponseMessage]) -> Option<Results> {
    msgs.iter().rev().find_map(|m| match m {
        JyankenResponseMessage::GameResults {
            weapons,
            scoreboard,
        } => Some((weapons.clone(), scoreboard.clone())),
        _ => None,
    })
}

fn flags(map: &[(&str, bool)]) -> HashMap<String, bool> {
    map.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

fn scores(map: &[(&str, (u32, u32))]) -> HashMap<String, (u32, u32)> {
    map.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

#[test]
fn each_weapon_beats_exactly_one_other() {
    let counts = HashMap::from([(Rock, 1), (Paper, 2), (Scissor, 4)]);
    assert_eq!(JyankenBehavior::calculate_wins(&Rock, &counts), 4);
    assert_eq!(JyankenBehavior::calculate_wins(&Paper, &counts), 1);
    assert_eq!(JyankenBehavior::calculate_wins(&Scissor, &counts), 2);
}

#[test]
fn no_wins_against_missing_weapons() {
    let counts = HashMap::from([(Rock, 3)]);
    assert_eq!(JyankenBehavior::calculate_wins(&Rock, &counts), 0);
    assert_eq!(JyankenBehavior::calculate_wins(&Paper, &counts), 3);
    assert_eq!(
        JyankenBehavior::calculate_wins(&Scissor, &HashMap::new()),
        0
    );
}

#[actix_rt::test]
async fn joining_user_gets_the_game_status_as_a_spectator() {
    let mut fixture = with_players(&["u1"]).await;

    fixture.join("u2").await;

    let msgs = jyanken(&fixture, "u2").await;
    assert_eq!(msgs.len(), 1);
    assert_eq!(
        last_status(&msgs),
        (flags(&[("u1", false)]), vec!["u2".to_string()])
    );
}

#[actix_rt::test]
async fn joining_the_game_is_broadcast() {
    let mut fixture = with_players(&["u1"]).await;
    fixture.join("u2").await;
    fixture.drain_all().await;

    fixture.send("u2", JOIN_GAME).await;

    for user_id in ["u1", "u2"] {
        let (selected, spectators) = last_status(&jyanken(&fixture, user_id).await);
        assert_eq!(selected, flags(&[("u1", false), ("u2", false)]));
        assert!(spectators.is_empty());
    }
}

#[actix_rt::test]
async fn joining_the_game_from_outside_the_room_is_ignored() {
    let fixture = with_players(&["u1"]).await;

    fixture.send("stranger", JOIN_GAME).await;

    assert!(fixture.take("u1").await.is_empty());
}

#[actix_rt::test]
async fn spectators_cannot_shoot() {
    let mut fixture = new_room();
    fixture.join("u1").await;
    fixture.drain_all().await;

    fixture.send("u1", &shoot(Rock)).await;

    assert_eq!(
        errors(&fixture.take("u1").await),
        vec!["You must join the game before shooting."]
    );
}

#[actix_rt::test]
async fn shot_is_confirmed_to_the_shooter_and_hidden_from_others() {
    let fixture = with_players(&["u1", "u2"]).await;

    fixture.send("u1", &shoot(Paper)).await;

    let own = jyanken(&fixture, "u1").await;
    assert!(matches!(
        &own[0],
        JyankenResponseMessage::OwnShoot { weapon: Paper }
    ));
    assert_eq!(last_status(&own).0, flags(&[("u1", true), ("u2", false)]));
    assert!(results(&own).is_none());

    let other = jyanken(&fixture, "u2").await;
    assert_eq!(other.len(), 1, "only the status, not the weapon");
}

#[actix_rt::test]
async fn results_and_scores_once_everyone_shot() {
    let fixture = with_players(&["u1", "u2"]).await;

    fixture.send("u1", &shoot(Rock)).await;
    fixture.send("u2", &shoot(Scissor)).await;

    for user_id in ["u1", "u2"] {
        let (weapons, scoreboard) = results(&jyanken(&fixture, user_id).await).unwrap();
        assert_eq!(
            weapons,
            HashMap::from([("u1".to_string(), Rock), ("u2".to_string(), Scissor)])
        );
        assert_eq!(scoreboard, scores(&[("u1", (1, 1)), ("u2", (0, 0))]));
    }
}

#[actix_rt::test]
async fn each_player_scores_one_win_per_beaten_opponent() {
    let fixture = with_players(&["a", "b", "c", "d"]).await;

    let scoreboard = play_round(
        &fixture,
        &[("a", Rock), ("b", Scissor), ("c", Scissor), ("d", Paper)],
    )
    .await;

    assert_eq!(
        scoreboard,
        scores(&[("a", (2, 2)), ("b", (1, 1)), ("c", (1, 1)), ("d", (1, 1))])
    );
}

#[actix_rt::test]
async fn ties_score_nothing() {
    let fixture = with_players(&["a", "b"]).await;

    let scoreboard = play_round(&fixture, &[("a", Paper), ("b", Paper)]).await;

    assert_eq!(scoreboard, scores(&[("a", (0, 0)), ("b", (0, 0))]));
}

#[actix_rt::test]
async fn scores_accumulate_across_rounds() {
    let fixture = with_players(&["a", "b"]).await;

    play_round(&fixture, &[("a", Rock), ("b", Scissor)]).await;
    fixture.send("a", NEW_GAME).await;
    play_round(&fixture, &[("a", Rock), ("b", Scissor)]).await;
    fixture.send("a", NEW_GAME).await;
    let scoreboard = play_round(&fixture, &[("a", Rock), ("b", Paper)]).await;

    // (wins last round, total wins)
    assert_eq!(scoreboard, scores(&[("a", (0, 2)), ("b", (1, 1))]));
}

#[actix_rt::test]
async fn changing_the_weapon_replaces_it_without_a_new_status() {
    let fixture = with_players(&["a", "b"]).await;
    fixture.send("a", &shoot(Rock)).await;
    fixture.drain_all().await;

    fixture.send("a", &shoot(Paper)).await;

    let own = jyanken(&fixture, "a").await;
    assert_eq!(own.len(), 1);
    assert!(fixture.take("b").await.is_empty());

    let scoreboard = play_round(&fixture, &[("b", Rock)]).await;
    assert_eq!(scoreboard, scores(&[("a", (1, 1)), ("b", (0, 0))]));
}

#[actix_rt::test]
async fn shooting_after_the_game_is_over_is_refused() {
    let fixture = with_players(&["a"]).await;
    play_round(&fixture, &[("a", Rock)]).await;

    fixture.send("a", &shoot(Paper)).await;

    let msgs = jyanken(&fixture, "a").await;
    assert_eq!(msgs.len(), 1);
    assert!(matches!(msgs[0], JyankenResponseMessage::GameOver));
}

#[actix_rt::test]
async fn new_game_resets_the_round_but_keeps_scores() {
    let fixture = with_players(&["a", "b"]).await;
    play_round(&fixture, &[("a", Rock), ("b", Scissor)]).await;

    fixture.send("b", NEW_GAME).await;

    for user_id in ["a", "b"] {
        let msgs = jyanken(&fixture, user_id).await;
        assert_eq!(msgs.len(), 1);
        assert!(matches!(msgs[0], JyankenResponseMessage::NewGame));
    }

    fixture.send("a", &shoot(Scissor)).await;
    let msgs = jyanken(&fixture, "a").await;
    assert_eq!(last_status(&msgs).0, flags(&[("a", true), ("b", false)]));
    assert!(results(&msgs).is_none());
}

#[actix_rt::test]
async fn new_game_from_outside_the_room_is_ignored() {
    let fixture = with_players(&["a"]).await;
    play_round(&fixture, &[("a", Rock)]).await;

    fixture.send("stranger", NEW_GAME).await;

    assert!(fixture.take("a").await.is_empty());
}

#[actix_rt::test]
async fn joining_after_the_game_ended_does_not_score_twice() {
    let mut fixture = with_players(&["a", "b"]).await;
    play_round(&fixture, &[("a", Rock), ("b", Scissor)]).await;
    fixture.join("c").await;

    // Joining a finished game re-sends the results, without recounting them.
    fixture.send("c", JOIN_GAME).await;

    let (_, scoreboard) = results(&jyanken(&fixture, "a").await).unwrap();
    assert_eq!(scoreboard, scores(&[("a", (1, 1)), ("b", (0, 0))]));
}

#[actix_rt::test]
async fn joining_after_the_game_ended_shows_the_results() {
    let mut fixture = with_players(&["a"]).await;
    play_round(&fixture, &[("a", Paper)]).await;

    fixture.join("late").await;

    let msgs = jyanken(&fixture, "late").await;
    assert_eq!(msgs.len(), 2);
    assert!(matches!(msgs[0], JyankenResponseMessage::GameStatus { .. }));
    assert!(results(&msgs).is_some());
}

#[actix_rt::test]
async fn removing_a_player_who_has_not_shot_ends_the_round() {
    let fixture = with_players(&["a", "b", "c"]).await;
    fixture.send("a", &shoot(Rock)).await;
    fixture.send("b", &shoot(Scissor)).await;
    fixture.drain_all().await;

    fixture.send("a", &remove("c")).await;

    let msgs = jyanken(&fixture, "c").await;
    let (selected, spectators) = last_status(&msgs);
    assert_eq!(selected, flags(&[("a", true), ("b", true)]));
    assert_eq!(spectators, vec!["c"]);
    let (weapons, _) = results(&msgs).unwrap();
    assert_eq!(weapons.len(), 2);
}

#[actix_rt::test]
async fn removed_player_cannot_shoot() {
    let fixture = with_players(&["a", "b"]).await;
    fixture.send("a", &remove("b")).await;
    fixture.send("a", NEW_GAME).await;
    fixture.drain_all().await;

    fixture.send("b", &shoot(Rock)).await;

    assert_eq!(
        errors(&fixture.take("b").await),
        vec!["You must join the game before shooting."]
    );
}

#[actix_rt::test]
async fn removing_a_spectator_does_nothing() {
    let mut fixture = with_players(&["a"]).await;
    fixture.join("b").await;
    fixture.drain_all().await;

    fixture.send("a", &remove("b")).await;

    assert!(fixture.take("a").await.is_empty());
}

#[actix_rt::test]
async fn removing_an_unknown_user_is_an_error() {
    let fixture = with_players(&["a", "b"]).await;

    fixture.send("a", &remove("ghost")).await;

    assert_eq!(errors(&fixture.take("a").await), vec!["User not found."]);
    assert!(fixture.take("b").await.is_empty());
}

#[actix_rt::test]
async fn leaving_player_loses_their_score() {
    let fixture = with_players(&["a", "b", "c"]).await;
    play_round(&fixture, &[("a", Rock), ("b", Scissor), ("c", Scissor)]).await;

    fixture.leave("a").await;

    let (weapons, scoreboard) = results(&jyanken(&fixture, "b").await).unwrap();
    assert!(!weapons.contains_key("a"));
    assert!(!scoreboard.contains_key("a"));
}

#[actix_rt::test]
async fn last_player_to_shoot_leaving_ends_the_round() {
    let fixture = with_players(&["a", "b"]).await;
    fixture.send("a", &shoot(Rock)).await;
    fixture.drain_all().await;

    fixture.leave("b").await;

    let (_, scoreboard) = results(&jyanken(&fixture, "a").await).unwrap();
    assert_eq!(scoreboard, scores(&[("a", (0, 0))]));
}

#[actix_rt::test]
async fn shots_from_outside_the_room_are_ignored() {
    let fixture = with_players(&["a"]).await;

    fixture.send("stranger", &shoot(Rock)).await;

    assert!(fixture.take("a").await.is_empty());
}
