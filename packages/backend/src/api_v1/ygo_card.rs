use axum::{Json, extract::State, response::IntoResponse};

use crate::api_v1::utils::Path;
use crate::prelude::*;
use crate::services::ygo_card;

/// Lists yugioh cards
pub async fn get_all(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let client = state.db.get().await?;

    let cards = ygo_card::get_all(&client).await?;

    Ok(Json(cards).into_response())
}

/// Get card by ID
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse> {
    let client = state.db.get().await?;

    let card = ygo_card::get_by_id(&client, id).await?;

    Ok(Json(card).into_response())
}

/// Import yugioh cards
pub async fn import(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let client = state.db.get().await?;

    let cards = ygo_card::import_sample_cards(&client).await?;

    Ok(Json(cards).into_response())
}

#[cfg(test)]
mod tests {
    use crate::models::ygo;
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
            // Seed sample cards
            let added_cards = {
                let client = state.db.get().await.expect("db");
                ygo_card::import_sample_cards(&client).await.expect("seed")
            };

            let router = Router::new()
                .route("/ygo/cards", get(get_all))
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
            assert_eq!(retrieved_cards.len(), added_cards.len());
        })
        .await
    }

    #[tokio::test]
    async fn test_get_yugioh_card() {
        with_app_state(async move |state| {
            // Seed sample cards
            {
                let client = state.db.get().await.expect("db");
                ygo_card::import_sample_cards(&client).await.expect("seed")
            };

            let router = Router::new()
                .route("/ygo/cards/{id}", get(get_by_id))
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
            assert_eq!(retrieved_card.data.name, "Blue-eyes White Dragon");
        })
        .await
    }
}
