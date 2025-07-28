use axum::{Json, extract::State, response::IntoResponse};

use crate::models::ygo;
use crate::prelude::*;

pub async fn get_yugioh_cards(State(_): State<AppState>) -> Result<impl IntoResponse> {
    let card1 = ygo::Card {
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
    };

    let card2 = ygo::Card {
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
    };

    let cards = vec![card1, card2];

    Ok(Json(cards).into_response())
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
                .route("/yugioh/cards", get(get_yugioh_cards))
                .with_state(state.as_ref().clone());

            let request = Request::builder()
                .uri("/yugioh/cards")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let retrieved_cards: Vec<ygo::Card> =
                serde_json::from_slice(&body).expect("Unable to parse response body");
            assert_eq!(retrieved_cards.len(), 2);
            assert_eq!(retrieved_cards.len(), 2);
        })
        .await
    }
}
