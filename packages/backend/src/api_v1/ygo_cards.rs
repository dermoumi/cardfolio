use axum::{Json, extract::State, response::IntoResponse};

use crate::api_v1::utils::Path;
use crate::models::ygo;
use crate::prelude::*;

fn make_card(id: i32) -> ygo::Card {
    match id {
        1 => ygo::Card {
            id: 1,
            updated_at: chrono::Utc::now(),
            data: ygo::CardData {
                name: "Blue-Eyes White Dragon".to_string(),
                description: "This legendary dragon is a powerful engine of destruction. Virtually invincible, very few have faced this awesome creature and lived to tell the tale.".to_string(),
                kind: ygo::CardKind::Monster,
                monster_attribute: Some(ygo::MonsterAttribute::Light),
                monster_race: Some(ygo::MonsterRace::Dragon),
                monster_level: Some(8),
                monster_atk: Some(3000),
                monster_def: Some(2500),
                ..Default::default()
            },
        },
        2 => ygo::Card {
            id: 2,
            updated_at: chrono::Utc::now(),
            data: ygo::CardData {
                name: "Dark Magician".to_string(),
                description: "The ultimate wizard in terms of attack and defense.".to_string(),
                kind: ygo::CardKind::Monster,
                monster_attribute: Some(ygo::MonsterAttribute::Dark),
                monster_race: Some(ygo::MonsterRace::Spellcaster),
                monster_level: Some(7),
                monster_atk: Some(2500),
                monster_def: Some(2100),
                ..Default::default()
            },
        },
        _ => {
            let name = format!("Card {}", id);
            let description = format!("This is the description for card {}.", id);
            let kind = match id % 5 {
                0 => ygo::CardKind::Spell,
                3 => ygo::CardKind::Trap,
                _ => ygo::CardKind::Monster,
            };

            let mut card_data = ygo::CardData {
                name,
                description,
                kind: kind.clone(),
                ..Default::default()
            };

            if kind == ygo::CardKind::Monster {
                // Use all 6 MonsterAttribute enum variants by mapping id % 6 to the variant
                card_data.monster_attribute = Some(match id % 6 {
                    0 => ygo::MonsterAttribute::Dark,
                    1 => ygo::MonsterAttribute::Light,
                    2 => ygo::MonsterAttribute::Earth,
                    3 => ygo::MonsterAttribute::Water,
                    4 => ygo::MonsterAttribute::Fire,
                    5 => ygo::MonsterAttribute::Wind,
                    _ => unreachable!(),
                });
                // Use all MonsterRace enum variants by mapping id to the variant
                card_data.monster_race = Some(match id % 8 {
                    0 => ygo::MonsterRace::Dragon,
                    1 => ygo::MonsterRace::Spellcaster,
                    2 => ygo::MonsterRace::Warrior,
                    3 => ygo::MonsterRace::Beast,
                    4 => ygo::MonsterRace::Fiend,
                    5 => ygo::MonsterRace::Fairy,
                    6 => ygo::MonsterRace::Zombie,
                    7 => ygo::MonsterRace::Machine,
                    _ => unreachable!(),
                });
                let variation_seed: i16 = id.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                card_data.monster_level = Some((variation_seed % 9) + 1);
                card_data.monster_atk = Some(1000 + (variation_seed % 20) * 100);
                card_data.monster_def = Some(800 + (variation_seed % 20) * 100);
            }

            ygo::Card {
                id,
                updated_at: chrono::Utc::now(),
                data: card_data,
            }
        }
    }
}

/// Lists yugioh cards
pub async fn get_ygo_cards(State(_): State<AppState>) -> Result<impl IntoResponse> {
    let cards: Vec<ygo::Card> = (1..=50).map(|id| make_card(id)).collect();

    Ok(Json(cards).into_response())
}

/// Get card by ID
pub async fn get_ygo_card(
    State(_): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let card = make_card(id);
    Ok(Json(card).into_response())
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;

    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };

    #[tokio::test]
    async fn test_get_yugioh_cards() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route("/ygo/cards", get(get_ygo_cards))
                .with_state(state.as_ref().clone());
            let request = Request::builder()
                .uri("/ygo/cards")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let retrieved_cards: Vec<ygo::Card> =
                serde_json::from_slice(&body).expect("Unable to parse response body");
            assert_eq!(retrieved_cards.len(), 50);
        })
        .await
    }

    #[tokio::test]
    async fn test_get_yugioh_card() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route("/ygo/cards/{id}", get(get_ygo_card))
                .with_state(state.as_ref().clone());
            let request = Request::builder()
                .uri("/ygo/cards/1")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let retrieved_card: ygo::Card =
                serde_json::from_slice(&body).expect("Unable to parse response body");
            assert_eq!(retrieved_card.id, 1);
            assert_eq!(retrieved_card.data.name, "Blue-Eyes White Dragon");
        })
        .await
    }
}
