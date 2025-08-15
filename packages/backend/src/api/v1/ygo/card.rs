use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::api::utils::{decode_pagination_cursor, encode_pagination_cursor};
use crate::api::{ApiError, ApiResult, Path, Query};
use crate::models::ygo;
use crate::prelude::AppState;
use crate::services::ygo as service;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    pub cards: Vec<crate::models::ygo::Card>,
    pub next: Option<String>,
}

/// Lists yugioh cards (paginated)
pub async fn get_cards(
    State(state): State<AppState>,
    Query(pagination): Query<Pagination>,
) -> ApiResult<impl IntoResponse> {
    let client = state.db.get().await?;

    let limit = pagination.limit.unwrap_or(100).min(100);
    let cursor = pagination
        .cursor
        .as_ref()
        .map(|c| decode_pagination_cursor(c))
        .transpose()?;

    let (cards, next_cursor) = service::card::get_page(&client, limit, cursor).await?;

    let as_page = Page {
        cards,
        next: next_cursor
            .as_ref()
            .map(encode_pagination_cursor)
            .transpose()?,
    };

    Ok(Json(as_page).into_response())
}

/// Get card by ID
pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> ApiResult<impl IntoResponse> {
    let client = state.db.get().await?;

    let card = service::card::get_by_id(&client, id)
        .await?
        .ok_or(ApiError::NotFound {
            resource: id.into(),
        })?;

    Ok(Json(card).into_response())
}

/// Import yugioh cards
pub async fn import(State(state): State<AppState>) -> ApiResult<impl IntoResponse> {
    let client = state.db.get().await?;

    let cards = service::card::seed_cards(&client, 80).await?;

    Ok(Json(cards).into_response())
}

/// Create a new card
pub async fn create(
    State(state): State<AppState>,
    Json(new_card): Json<ygo::NewCard>,
) -> ApiResult<impl IntoResponse> {
    let client = state.db.get().await?;
    let created = service::card::save_new(&client, &new_card).await?;
    Ok((StatusCode::CREATED, Json(created)).into_response())
}

/// Delete a card by ID
pub async fn delete_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> ApiResult<impl IntoResponse> {
    let client = state.db.get().await?;

    let deleted = service::card::delete_by_id(&client, id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound {
            resource: id.into(),
        })
    }
}

/// Update a card by ID
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(data): Json<ygo::CardData>,
) -> ApiResult<impl IntoResponse> {
    let client = state.db.get().await?;

    // Build a Card for the service layer. updated_at is ignored by SQL (set in DB),
    // but the struct requires a value.
    let to_update = ygo::Card {
        id,
        updated_at: chrono::Utc::now(),
        data,
    };

    let updated = service::card::save(&client, &to_update)
        .await?
        .ok_or(ApiError::NotFound {
            resource: id.into(),
        })?;

    Ok(Json(updated).into_response())
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
        routing::{delete, get, post, put},
    };

    #[tokio::test]
    async fn test_get_cards() {
        with_app_state(async move |state| {
            // Seed sample cards
            let added_cards = {
                let client = state.db.get().await.expect("db");
                service::card::seed_cards(&client, 1).await.expect("seed")
            };

            let router = Router::new()
                .route("/ygo/cards", get(get_cards))
                .with_state(state.as_ref().clone());
            let request = Request::builder()
                .uri("/ygo/cards")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let paginated: super::Page =
                serde_json::from_slice(&body).expect("Unable to parse response body");
            assert_eq!(paginated.cards.len(), added_cards.len());
        })
        .await
    }

    #[tokio::test]
    async fn test_get_by_id() {
        with_app_state(async move |state| {
            // Seed sample cards
            {
                let client = state.db.get().await.expect("db");
                service::card::seed_cards(&client, 1).await.expect("seed")
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
            assert_eq!(retrieved_card.data.name, "Blue-Eyes White Dragon");
        })
        .await
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route("/ygo/cards/{id}", get(get_by_id))
                .with_state(state.as_ref().clone());
            let request = Request::builder()
                .uri("/ygo/cards/144")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::NOT_FOUND);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(body, r#"{"error":"not_found","resource":144}"#);
        })
        .await
    }

    #[tokio::test]
    async fn test_get_cards_invalid_query() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route("/ygo/cards", get(get_cards))
                .with_state(state.as_ref().clone());
            let request = Request::builder()
                .uri("/ygo/cards?limit=invalid")
                .body(Body::empty())
                .unwrap();

            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(
                body,
                r#"{"error":"query_error","message":"Failed to deserialize query string: limit: invalid digit found in string"}"#
            );
        })
        .await
    }

    #[tokio::test]
    async fn test_create_card_success() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route("/ygo/cards", post(create))
                .route("/ygo/cards/{id}", get(get_by_id))
                .with_state(state.as_ref().clone());

            let new = ygo::NewCard {
                data: ygo::CardData {
                    name: "Test Monster".to_string(),
                    description: "A test card".to_string(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_attribute: Some(ygo::MonsterAttribute::Light),
                    monster_race: Some(ygo::MonsterRace::Dragon),
                    monster_level: Some(4),
                    monster_atk: Some(1500),
                    monster_def: Some(1200),
                    ..Default::default()
                },
            };

            let body = serde_json::to_vec(&new).unwrap();
            let request = Request::builder()
                .method("POST")
                .uri("/ygo/cards")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap();

            let response = router.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::CREATED);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let created: ygo::Card = serde_json::from_slice(&body).expect("json");
            assert!(created.id > 0);
            assert_eq!(created.data.name, "Test Monster");

            // Fetch it back
            let get = Request::builder()
                .uri(format!("/ygo/cards/{}", created.id))
                .body(Body::empty())
                .unwrap();
            let resp = router.oneshot(get).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        })
        .await
    }

    #[tokio::test]
    async fn test_delete_by_id_success() {
        with_app_state(async move |state| {
            // Seed
            {
                let client = state.db.get().await.expect("db");
                service::card::seed_cards(&client, 1).await.expect("seed");
            }

            let router = Router::new()
                .route("/ygo/cards/{id}", delete(delete_by_id).get(get_by_id))
                .with_state(state.as_ref().clone());

            // Delete
            let del = Request::builder()
                .method("DELETE")
                .uri("/ygo/cards/1")
                .body(Body::empty())
                .unwrap();
            let resp = router.clone().oneshot(del).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NO_CONTENT);

            // Verify gone
            let get = Request::builder()
                .uri("/ygo/cards/1")
                .body(Body::empty())
                .unwrap();
            let resp = router.oneshot(get).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        })
        .await
    }

    #[tokio::test]
    async fn test_delete_by_id_not_found() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route("/ygo/cards/{id}", delete(delete_by_id))
                .with_state(state.as_ref().clone());

            let del = Request::builder()
                .method("DELETE")
                .uri("/ygo/cards/9999")
                .body(Body::empty())
                .unwrap();
            let resp = router.oneshot(del).await.unwrap();
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);

            let body = resp.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(body, r#"{"error":"not_found","resource":9999}"#);
        })
        .await
    }

    #[tokio::test]
    async fn test_update_card_success() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route("/ygo/cards", post(create))
                .route("/ygo/cards/{id}", get(get_by_id).put(update))
                .with_state(state.as_ref().clone());

            // Create a card first
            let new = ygo::NewCard {
                data: ygo::CardData {
                    name: "Updatable".to_string(),
                    description: "Before".to_string(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_attribute: Some(ygo::MonsterAttribute::Light),
                    monster_race: Some(ygo::MonsterRace::Dragon),
                    monster_level: Some(4),
                    monster_atk: Some(1400),
                    monster_def: Some(1200),
                    ..Default::default()
                },
            };
            let body = serde_json::to_vec(&new).unwrap();
            let create_req = Request::builder()
                .method("POST")
                .uri("/ygo/cards")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap();
            let created_resp = router.clone().oneshot(create_req).await.unwrap();
            assert_eq!(created_resp.status(), StatusCode::CREATED);
            let created: ygo::Card = serde_json::from_slice(
                &created_resp.into_body().collect().await.unwrap().to_bytes(),
            )
            .unwrap();

            // Prepare update payload (CardData)
            let mut data = created.data.clone();
            data.name = "Updated Name".to_string();
            data.description = "After".to_string();
            data.monster_atk = Some(1600);

            // Send PUT
            let upd_body = serde_json::to_vec(&data).unwrap();
            let update_req = Request::builder()
                .method("PUT")
                .uri(format!("/ygo/cards/{}", created.id))
                .header("content-type", "application/json")
                .body(Body::from(upd_body))
                .unwrap();
            let update_resp = router.clone().oneshot(update_req).await.unwrap();
            assert_eq!(update_resp.status(), StatusCode::OK);
            let updated: ygo::Card = serde_json::from_slice(
                &update_resp.into_body().collect().await.unwrap().to_bytes(),
            )
            .unwrap();
            assert_eq!(updated.id, created.id);
            assert_eq!(updated.data.name, "Updated Name");
            assert_eq!(updated.data.description, "After");
            assert_eq!(updated.data.monster_atk, Some(1600));

            // GET to verify
            let get_req = Request::builder()
                .uri(format!("/ygo/cards/{}", created.id))
                .body(Body::empty())
                .unwrap();
            let get_resp = router.oneshot(get_req).await.unwrap();
            assert_eq!(get_resp.status(), StatusCode::OK);
        })
        .await
    }

    #[tokio::test]
    async fn test_update_card_not_found() {
        with_app_state(async move |state| {
            let router = Router::new()
                .route("/ygo/cards/{id}", put(update))
                .with_state(state.as_ref().clone());

            let payload = ygo::CardData {
                name: "Doesn't matter".to_string(),
                description: "Missing".to_string(),
                kind: ygo::CardKind::Monster,
                monster_kind: Some(ygo::MonsterKind::Normal),
                monster_attribute: Some(ygo::MonsterAttribute::Light),
                monster_race: Some(ygo::MonsterRace::Dragon),
                monster_level: Some(4),
                monster_atk: Some(1400),
                monster_def: Some(1200),
                ..Default::default()
            };
            let body = serde_json::to_vec(&payload).unwrap();
            let request = Request::builder()
                .method("PUT")
                .uri("/ygo/cards/9999")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap();
            let response = router.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
            let body = response.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(body, r#"{"error":"not_found","resource":9999}"#);
        })
        .await
    }
}
