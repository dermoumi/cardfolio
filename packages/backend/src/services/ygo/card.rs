use serde::{Deserialize, Serialize};
use std::result::Result;
use tokio_postgres::{Client, Error, Row};

use crate::database::{QueryParams, TzTimestamp};
use crate::models::ygo;

#[derive(Debug, Serialize, Deserialize)]
pub struct PageCursor {
    pub id: i32,
}

#[derive(Debug, Deserialize)]
pub struct Filter {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub kind: Option<ygo::CardKind>,
    #[serde(default)]
    pub attribute: Vec<ygo::MonsterAttribute>,
    #[serde(default)]
    pub race: Vec<ygo::MonsterRace>,
    #[serde(default)]
    pub subtype: Vec<ygo::MonsterSubtype>,
    pub atk_min: Option<i32>,
    pub atk_max: Option<i32>,
    pub def_min: Option<i32>,
    pub def_max: Option<i32>,
    pub level_min: Option<i32>,
    pub level_max: Option<i32>,
    #[serde(default)]
    pub spell: Vec<ygo::SpellKind>,
    #[serde(default)]
    pub trap: Vec<ygo::TrapKind>,
}

/// Retrieves cards with cursor-based pagination
pub async fn get_page(
    client: &Client,
    filter: Option<Filter>,
    limit: u32,
    cursor: Option<PageCursor>,
) -> Result<(Vec<ygo::Card>, Option<PageCursor>), Error> {
    // Tiny query builder
    let mut query = String::from("SELECT * FROM ygo_cards");
    let mut params = QueryParams::new();
    let mut where_queries: Vec<String> = Vec::new();

    if let Some(filter) = filter {
        // Filter by name
        if let Some(name) = filter.name {
            let idx = params.push(format!("%{}%", name));
            where_queries.push(format!("name ILIKE ${idx}"));
        }

        // Filter by description
        if let Some(description) = filter.description {
            let idx = params.push(format!("%{}%", description));
            where_queries.push(format!("description ILIKE ${idx}"));
        }

        // Filter by kind
        if let Some(kind) = filter.kind {
            let idx = params.push(kind);
            where_queries.push(format!("kind = ${idx}"));
        }

        // Filter by attribute
        if !filter.attribute.is_empty() {
            let idx = params.push(filter.attribute);
            where_queries.push(format!("monster_attribute = ANY(${idx})"));
        }

        // Filter by race
        if !filter.race.is_empty() {
            let idx = params.push(filter.race);
            where_queries.push(format!("monster_race = ANY(${idx})"));
        }

        // Filter by subtype
        if !filter.subtype.is_empty() {
            let idx = params.push(filter.subtype);
            where_queries.push(format!("monster_subtypes = ANY(${idx})"));
        }

        // Filter by attack points
        if let Some(atk_min) = filter.atk_min {
            let idx = params.push(atk_min);
            where_queries.push(format!("monster_atk >= ${idx}"));
        }
        if let Some(atk_max) = filter.atk_max {
            let idx = params.push(atk_max);
            where_queries.push(format!("monster_atk <= ${idx}"));
        }

        // Filter by defense points
        if let Some(def_min) = filter.def_min {
            let idx = params.push(def_min);
            where_queries.push(format!("monster_def >= ${idx}"));
        }
        if let Some(def_max) = filter.def_max {
            let idx = params.push(def_max);
            where_queries.push(format!("monster_def <= ${idx}"));
        }

        // Filter by level
        if let Some(level_min) = filter.level_min {
            let idx = params.push(level_min);
            where_queries.push(format!("monster_level >= ${idx}"));
        }
        if let Some(level_max) = filter.level_max {
            let idx = params.push(level_max);
            where_queries.push(format!("monster_level <= ${idx}"));
        }

        // Filter by spell kind
        if !filter.spell.is_empty() {
            let idx = params.push(filter.spell);
            where_queries.push(format!("spell_kind = ANY(${idx})"));
        }

        // Filter by trap kind
        if !filter.trap.is_empty() {
            let idx = params.push(filter.trap);
            where_queries.push(format!("trap_kind = ANY(${idx})"));
        }
    }

    // Retrieve only items after the cursor index
    if let Some(PageCursor { id }) = cursor {
        let idx = params.push(id);
        where_queries.push(format!("id > ${idx}"));
    }

    // Build the where queries
    if !where_queries.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&where_queries.join(" AND "));
    }

    // Retrieve one extra item to check if there's still another page
    let limit_plus = (limit + 1) as i64;
    let idx = params.push(limit_plus);
    query.push_str(&format!(" ORDER BY id ASC LIMIT ${idx}"));

    // Make the query
    let rows = client.query(&query, &params.as_refs()).await?;

    // Retrieve the cards
    let cards: Vec<ygo::Card> = rows
        .iter()
        .take(limit as usize)
        .map(|row| row.try_into())
        .collect::<Result<_, _>>()?;

    // Generate the next cursor
    let next_cursor = if rows.len() > limit as usize {
        cards.last().map(|card| PageCursor { id: card.id })
    } else {
        None
    };

    Ok((cards, next_cursor))
}

/// Retrieves all cards in the database
#[cfg(test)]
pub async fn get_all(client: &Client) -> Result<Vec<ygo::Card>, Error> {
    let query = "SELECT * FROM ygo_cards ORDER BY id ASC";
    let rows = client.query(query, &[]).await?;

    let cards: Vec<ygo::Card> = rows
        .iter()
        .map(|row| row.try_into())
        .collect::<Result<_, _>>()?;

    Ok(cards)
}

/// Retrieves a card by ID
pub async fn get_by_id(client: &Client, id: i32) -> Result<Option<ygo::Card>, Error> {
    let query = "SELECT * FROM ygo_cards WHERE id = $1";
    let row = &client.query_opt(query, &[&id]).await?;

    row.as_ref().map(|r| r.try_into()).transpose()
}

/// Retrieves a card by Konami ID
pub async fn get_by_konami_id(client: &Client, konami_id: i32) -> Result<Option<ygo::Card>, Error> {
    let query = "SELECT * FROM ygo_cards WHERE konami_id = $1";
    let row = &client.query_opt(query, &[&konami_id]).await?;

    row.as_ref().map(|r| r.try_into()).transpose()
}

/// Retrieves a card by password
pub async fn get_by_password(client: &Client, password: &str) -> Result<Option<ygo::Card>, Error> {
    let query = "SELECT * FROM ygo_cards WHERE password = $1";
    let row = &client.query_opt(query, &[&password]).await?;

    row.as_ref().map(|r| r.try_into()).transpose()
}

/// Deletes a card by ID. Returns true if a row was deleted, false otherwise.
pub async fn delete_by_id(client: &Client, id: i32) -> Result<bool, Error> {
    let affected = client
        .execute("DELETE FROM ygo_cards WHERE id = $1", &[&id])
        .await?;
    Ok(affected == 1)
}

/// Insert a new card and return the created record.
pub async fn save_new(client: &Client, new_card: &ygo::NewCard) -> Result<ygo::Card, Error> {
    let card_data = &new_card.data;
    let row = client
        .query_one(
            r#"
            INSERT INTO ygo_cards (
                name,
                description,
                kind,
                password,
                konami_id,
                treated_as,
                tcg_date,
                ocg_date,
                monster_kind,
                monster_attribute,
                monster_race,
                monster_subtypes,
                monster_atk,
                monster_def,
                monster_level,
                monster_pendulum_scale,
                monster_pendulum_effect,
                monster_link_arrows,
                spell_kind,
                trap_kind
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
            ) RETURNING *
            "#,
            &[
                &card_data.name,
                &card_data.description,
                &card_data.kind,
                &card_data.password,
                &card_data.konami_id,
                &card_data.treated_as,
                &card_data.tcg_date,
                &card_data.ocg_date,
                &card_data.monster_kind,
                &card_data.monster_attribute,
                &card_data.monster_race,
                &card_data.monster_subtypes,
                &card_data.monster_atk,
                &card_data.monster_def,
                &card_data.monster_level,
                &card_data.monster_pendulum_scale,
                &card_data.monster_pendulum_effect,
                &card_data.monster_link_arrows,
                &card_data.spell_kind,
                &card_data.trap_kind,
            ],
        )
        .await?;

    let card: ygo::Card = (&row).try_into()?;
    Ok(card)
}

/// Update an existing card by id and return the updated record.
pub async fn save(client: &Client, card: &ygo::Card) -> Result<Option<ygo::Card>, Error> {
    let id = card.id;
    let d = &card.data;

    let row = client
        .query_opt(
            r#"
            UPDATE ygo_cards SET
                name = $1,
                description = $2,
                kind = $3,
                password = $4,
                konami_id = $5,
                treated_as = $6,
                tcg_date = $7,
                ocg_date = $8,
                monster_kind = $9,
                monster_attribute = $10,
                monster_race = $11,
                monster_subtypes = $12,
                monster_atk = $13,
                monster_def = $14,
                monster_level = $15,
                monster_pendulum_scale = $16,
                monster_pendulum_effect = $17,
                monster_link_arrows = $18,
                spell_kind = $19,
                trap_kind = $20,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $21
            RETURNING *
            "#,
            &[
                &d.name,
                &d.description,
                &d.kind,
                &d.password,
                &d.konami_id,
                &d.treated_as,
                &d.tcg_date,
                &d.ocg_date,
                &d.monster_kind,
                &d.monster_attribute,
                &d.monster_race,
                &d.monster_subtypes,
                &d.monster_atk,
                &d.monster_def,
                &d.monster_level,
                &d.monster_pendulum_scale,
                &d.monster_pendulum_effect,
                &d.monster_link_arrows,
                &d.spell_kind,
                &d.trap_kind,
                &id,
            ],
        )
        .await?;

    row.as_ref().map(|r| r.try_into()).transpose()
}

#[cfg(test)]
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
                card_data.monster_race = Some(match id % 4 {
                    0 => ygo::MonsterRace::Dragon,
                    1 => ygo::MonsterRace::Spellcaster,
                    2 => ygo::MonsterRace::Warrior,
                    3 => ygo::MonsterRace::Beast,
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

/// Seeds the database with a given number of sample Yu-Gi-Oh! cards.
/// Used by the import HTTP handler and tests.
#[cfg(test)]
pub async fn seed_cards(client: &Client, amount: usize) -> Result<Vec<ygo::Card>, Error> {
    let mut cards = Vec::with_capacity(amount);

    for id in 1..=amount {
        let id = id as i32;
        let card = make_card(id);
        let d = &card.data;

        // Try to insert; if it already exists, fetch the existing row.
        let inserted = client
            .query_opt(
                r#"
                INSERT INTO ygo_cards (
                    id,
                    name,
                    description,
                    kind,
                    password,
                    konami_id,
                    treated_as,
                    tcg_date,
                    ocg_date,
                    monster_kind,
                    monster_attribute,
                    monster_race,
                    monster_subtypes,
                    monster_atk,
                    monster_def,
                    monster_level,
                    monster_pendulum_scale,
                    monster_pendulum_effect,
                    monster_link_arrows,
                    spell_kind,
                    trap_kind
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19,
                    $20, $21
                )
                ON CONFLICT (id) DO NOTHING
                RETURNING *
                "#,
                &[
                    &card.id,
                    &d.name,
                    &d.description,
                    &d.kind,
                    &d.password,
                    &d.konami_id,
                    &d.treated_as,
                    &d.tcg_date,
                    &d.ocg_date,
                    &d.monster_kind,
                    &d.monster_attribute,
                    &d.monster_race,
                    &d.monster_subtypes,
                    &d.monster_atk,
                    &d.monster_def,
                    &d.monster_level,
                    &d.monster_pendulum_scale,
                    &d.monster_pendulum_effect,
                    &d.monster_link_arrows,
                    &d.spell_kind,
                    &d.trap_kind,
                ],
            )
            .await?;

        let card_row = if let Some(row) = inserted {
            row
        } else {
            client
                .query_one("SELECT * FROM ygo_cards WHERE id = $1", &[&id])
                .await?
        };

        let saved: ygo::Card = (&card_row).try_into()?;
        cards.push(saved);
    }

    Ok(cards)
}

impl TryFrom<&Row> for ygo::Card {
    type Error = Error;

    /// Converts a database row into a YugiohCard struct
    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let id: i32 = value.get("id");
        let updated_at: TzTimestamp = value.get("updated_at");

        Ok(Self {
            id,
            updated_at: updated_at.0,
            data: value.try_into()?,
        })
    }
}

impl TryFrom<&Row> for ygo::CardData {
    type Error = Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.try_get("name")?,
            description: value.try_get("description")?,
            kind: value.try_get("kind")?,
            password: value.try_get("password")?,
            konami_id: value.try_get("konami_id")?,
            treated_as: value.try_get("treated_as")?,
            tcg_date: value.try_get("tcg_date")?,
            ocg_date: value.try_get("ocg_date")?,
            monster_kind: value.try_get("monster_kind")?,
            monster_attribute: value.try_get("monster_attribute")?,
            monster_race: value.try_get("monster_race")?,
            monster_subtypes: value.try_get("monster_subtypes")?,
            monster_atk: value.try_get("monster_atk")?,
            monster_def: value.try_get("monster_def")?,
            monster_level: value.try_get("monster_level")?,
            monster_pendulum_scale: value.try_get("monster_pendulum_scale")?,
            monster_pendulum_effect: value.try_get("monster_pendulum_effect")?,
            monster_link_arrows: value.try_get("monster_link_arrows")?,
            spell_kind: value.try_get("spell_kind")?,
            trap_kind: value.try_get("trap_kind")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::ygo, test_utils::with_db_pool};

    #[tokio::test]
    async fn test_get_page_basic_pagination() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");
            // Seed 15 cards
            let _ = seed_cards(&client, 15).await.expect("seed");

            // Get first page (limit 5)
            let (page1, next1) = get_page(&client, None, 5, None).await.expect("page1");
            assert_eq!(page1.len(), 5);
            assert!(next1.is_some());

            // Get second page
            let (page2, next2) = get_page(&client, None, 5, next1).await.expect("page2");
            assert_eq!(page2.len(), 5);
            assert!(next2.is_some());

            // Get third page (should have 5 or less)
            let (page3, next3) = get_page(&client, None, 5, next2).await.expect("page3");
            assert_eq!(page3.len(), 5);
            assert!(next3.is_none());
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_page_empty_and_exact() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");
            // No cards in DB
            let (empty, next) = get_page(&client, None, 10, None).await.expect("empty");
            assert!(empty.is_empty());
            assert!(next.is_none());

            // Seed exactly 3 cards
            let _ = seed_cards(&client, 3).await.expect("seed");
            let (all, next) = get_page(&client, None, 10, None).await.expect("all");
            assert_eq!(all.len(), 3);
            assert!(next.is_none());
        })
        .await;
    }

    #[tokio::test]
    async fn test_save_new_inserts_and_returns_card() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

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

            let created = save_new(&client, &new).await.expect("insert");
            assert!(created.id > 0);
            assert_eq!(created.data.name, new.data.name);
            assert_eq!(created.data.kind, ygo::CardKind::Monster);

            let fetched = get_by_id(&client, created.id).await.expect("fetch");
            assert_eq!(fetched, Some(created));
        })
        .await;
    }

    #[tokio::test]
    async fn test_save_updates_card() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            // Create
            let new = ygo::NewCard {
                data: ygo::CardData {
                    name: "To Be Updated".to_string(),
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
            let created = save_new(&client, &new).await.expect("insert");

            // Modify a few fields
            let mut to_update = created.clone();
            to_update.data.name = "Updated Name".to_string();
            to_update.data.description = "After".to_string();
            to_update.data.monster_atk = Some(1600);

            // Save
            let updated = save(&client, &to_update).await.expect("save");
            assert!(updated.is_some());
            if let Some(updated) = updated {
                assert_eq!(updated.id, created.id);
                assert_eq!(updated.data.name, "Updated Name");
                assert_eq!(updated.data.description, "After");
                assert_eq!(updated.data.monster_atk, Some(1600));
            }

            // Fetch to confirm persistence
            let fetched = get_by_id(&client, created.id).await.expect("fetch");
            assert!(fetched.is_some());
            if let Some(fetched) = fetched {
                assert_eq!(fetched.data.name, "Updated Name");
                assert_eq!(fetched.data.description, "After");
                assert_eq!(fetched.data.monster_atk, Some(1600));
            }
        })
        .await;
    }
}
