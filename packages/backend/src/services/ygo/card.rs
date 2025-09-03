use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::result::Result;
use tokio_postgres::{Client, Error, Row};

use crate::database::{QueryParams, TzTimestamp};
use crate::models::ygo;

#[derive(Debug, Serialize, Deserialize)]
pub struct PageCursor {
    pub id: i32,
    pub sorting_value: Option<SortingCursor>,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortingField {
    #[default]
    Id,
    Name,
    Atk,
    Def,
    Level,
    TcgDate,
    OcgDate,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortingDirection {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Default, Deserialize)]
pub struct Sort {
    #[serde(default)]
    pub sort: SortingField,
    #[serde(default)]
    pub dir: SortingDirection,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SortingCursor {
    Name(String),
    Int(i16),
    Date(NaiveDate),
}

#[derive(Debug, Default, Deserialize)]
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
    pub atk_min: Option<i16>,
    pub atk_max: Option<i16>,
    pub def_min: Option<i16>,
    pub def_max: Option<i16>,
    pub level_min: Option<i16>,
    pub level_max: Option<i16>,
    #[serde(default)]
    pub spell: Vec<ygo::SpellKind>,
    #[serde(default)]
    pub trap: Vec<ygo::TrapKind>,
}

/// Retrieves cards with cursor-based pagination
pub async fn get_page(
    client: &Client,
    filter: Option<Filter>,
    sort: Option<Sort>,
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
            where_queries.push(format!("monster_subtypes @> ${idx}"));
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

    let sort = sort.unwrap_or_default();
    let (sort_dir, sort_comparator) = match sort.dir {
        SortingDirection::Asc => ("ASC", ">"),
        SortingDirection::Desc => ("DESC", "<"),
    };
    let sorting_field = match sort.sort {
        SortingField::Id => "id",
        SortingField::Name => "name",
        SortingField::Atk => "monster_atk",
        SortingField::Def => "monster_def",
        SortingField::Level => "monster_level",
        SortingField::TcgDate => "tcg_date",
        SortingField::OcgDate => "ocg_date",
    };

    // Retrieve only items after the cursor index
    if let Some(PageCursor { id, sorting_value }) = cursor {
        let id_idx = params.push(id);

        if let Some(value) = sorting_value
            && sort.sort != SortingField::Id
        {
            let value_idx = match value {
                SortingCursor::Name(name) => params.push(name),
                SortingCursor::Int(int) => params.push(int),
                SortingCursor::Date(date) => params.push(date),
            };

            where_queries.push(format!("{sorting_field} {sort_comparator} ${value_idx} OR ({sorting_field} = ${value_idx} AND id {sort_comparator} ${id_idx})"));
        } else {
            where_queries.push(format!("id > ${id_idx}"));
        }
    }

    // Build the where queries
    if !where_queries.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&where_queries.join(" AND "));
    }

    // Sort by both the sorting field and ID
    if sort.sort == SortingField::Id {
        query.push_str(&format!(" ORDER BY id {sort_dir}"));
    } else {
        query.push_str(&format!(
            " ORDER BY {sorting_field} {sort_dir}, id {sort_dir}"
        ));
    }

    // Retrieve one extra item to check if there's still another page
    let idx = params.push((limit + 1) as i64);
    query.push_str(&format!(" LIMIT ${idx}"));

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
        cards.last().map(move |card| {
            let sorting_value = match sort.sort {
                SortingField::Id => None,
                SortingField::Name => Some(SortingCursor::Name(card.data.name.clone())),
                SortingField::Atk => card.data.monster_atk.map(SortingCursor::Int),
                SortingField::Def => card.data.monster_def.map(SortingCursor::Int),
                SortingField::Level => card.data.monster_level.map(SortingCursor::Int),
                SortingField::TcgDate => card.data.tcg_date.map(SortingCursor::Date),
                SortingField::OcgDate => card.data.ocg_date.map(SortingCursor::Date),
            };

            PageCursor {
                id: card.id,
                sorting_value,
            }
        })
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
                trap_kind,
                ygoprodeck_id
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21
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
                &card_data.ygoprodeck_id
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
                ygoprodeck_id = $21,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $22
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
                &d.ygoprodeck_id,
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
                    trap_kind,
                    ygoprodeck_id
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                    $11, $12, $13, $14, $15, $16, $17, $18, $19,
                    $20, $21, $22
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
                    &d.ygoprodeck_id,
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
            ygoprodeck_id: value.try_get("ygoprodeck_id")?,
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
            let (page1, next1) = get_page(&client, None, None, 5, None).await.expect("page1");
            assert_eq!(page1.len(), 5);
            assert!(next1.is_some());

            // Get second page
            let (page2, next2) = get_page(&client, None, None, 5, next1)
                .await
                .expect("page2");
            assert_eq!(page2.len(), 5);
            assert!(next2.is_some());

            // Get third page (should have 5 or less)
            let (page3, next3) = get_page(&client, None, None, 5, next2)
                .await
                .expect("page3");
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
            let (empty, next) = get_page(&client, None, None, 10, None)
                .await
                .expect("empty");
            assert!(empty.is_empty());
            assert!(next.is_none());

            // Seed exactly 3 cards
            let _ = seed_cards(&client, 3).await.expect("seed");
            let (all, next) = get_page(&client, None, None, 10, None).await.expect("all");
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

    #[tokio::test]
    async fn test_filter_by_name() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let c_ok = ygo::NewCard {
                data: ygo::CardData {
                    name: "Blue-Eyes White Dragon".into(),
                    description: "legendary dragon".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    ..Default::default()
                },
            };
            let c_bad = ygo::NewCard {
                data: ygo::CardData {
                    name: "Dark Magician".into(),
                    description: "ultimate wizard".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    ..Default::default()
                },
            };
            let c_ok = save_new(&client, &c_ok).await.unwrap();
            let c_bad = save_new(&client, &c_bad).await.unwrap();

            let mut filter = Filter::default();
            filter.name = Some("blue-eyes".into());
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(
                cards
                    .iter()
                    .all(|c| c.data.name.to_lowercase().contains("blue"))
            );
            assert!(!cards.iter().any(|c| c.id == c_bad.id));
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_description() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let c_ok = ygo::NewCard {
                data: ygo::CardData {
                    name: "Alpha".into(),
                    description: "Powerful engine of destruction".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    ..Default::default()
                },
            };
            let c_bad = ygo::NewCard {
                data: ygo::CardData {
                    name: "Beta".into(),
                    description: "Gentle creature".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    ..Default::default()
                },
            };
            let c_ok = save_new(&client, &c_ok).await.unwrap();
            let c_bad = save_new(&client, &c_bad).await.unwrap();

            let mut filter = Filter::default();
            filter.description = Some("engine of destruction".into());
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(
                cards
                    .iter()
                    .all(|c| c.data.description.to_lowercase().contains("engine"))
            );
            assert!(!cards.iter().any(|c| c.id == c_bad.id));
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_kind() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let c_ok = ygo::NewCard {
                data: ygo::CardData {
                    name: "Mystical Space Typhoon".into(),
                    description: "Destroy 1 spell/trap".into(),
                    kind: ygo::CardKind::Spell,
                    spell_kind: Some(ygo::SpellKind::QuickPlay),
                    ..Default::default()
                },
            };
            let c_bad = ygo::NewCard {
                data: ygo::CardData {
                    name: "Generic Monster".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    ..Default::default()
                },
            };
            let c_ok = save_new(&client, &c_ok).await.unwrap();
            let c_bad = save_new(&client, &c_bad).await.unwrap();

            let mut filter = Filter::default();
            filter.kind = Some(ygo::CardKind::Spell);
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(cards.iter().all(|c| c.data.kind == ygo::CardKind::Spell));
            assert!(!cards.iter().any(|c| c.id == c_bad.id));
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_attribute() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let c_ok = ygo::NewCard {
                data: ygo::CardData {
                    name: "Light Monster".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_attribute: Some(ygo::MonsterAttribute::Light),
                    ..Default::default()
                },
            };
            let c_bad = ygo::NewCard {
                data: ygo::CardData {
                    name: "Dark Monster".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_attribute: Some(ygo::MonsterAttribute::Dark),
                    ..Default::default()
                },
            };
            let c_ok = save_new(&client, &c_ok).await.unwrap();
            let c_bad = save_new(&client, &c_bad).await.unwrap();

            let mut filter = Filter::default();
            filter.attribute = vec![ygo::MonsterAttribute::Light];
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(
                cards
                    .iter()
                    .all(|c| c.data.monster_attribute == Some(ygo::MonsterAttribute::Light))
            );
            assert!(!cards.iter().any(|c| c.id == c_bad.id));
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_race() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let c_ok = ygo::NewCard {
                data: ygo::CardData {
                    name: "Dragon".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_race: Some(ygo::MonsterRace::Dragon),
                    ..Default::default()
                },
            };
            let c_bad = ygo::NewCard {
                data: ygo::CardData {
                    name: "Warrior".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_race: Some(ygo::MonsterRace::Warrior),
                    ..Default::default()
                },
            };
            let c_ok = save_new(&client, &c_ok).await.unwrap();
            let c_bad = save_new(&client, &c_bad).await.unwrap();

            let mut filter = Filter::default();
            filter.race = vec![ygo::MonsterRace::Dragon];
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(
                cards
                    .iter()
                    .all(|c| c.data.monster_race == Some(ygo::MonsterRace::Dragon))
            );
            assert!(!cards.iter().any(|c| c.id == c_bad.id));
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_subtype() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let c_ok = ygo::NewCard {
                data: ygo::CardData {
                    name: "Flip Monster".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Effect),
                    monster_subtypes: Some(vec![ygo::MonsterSubtype::Flip]),
                    ..Default::default()
                },
            };
            let c_bad = ygo::NewCard {
                data: ygo::CardData {
                    name: "Tuner Monster".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Effect),
                    monster_subtypes: Some(vec![ygo::MonsterSubtype::Tuner]),
                    ..Default::default()
                },
            };
            let c_ok = save_new(&client, &c_ok).await.unwrap();
            let c_bad = save_new(&client, &c_bad).await.unwrap();

            let mut filter = Filter::default();
            filter.subtype = vec![ygo::MonsterSubtype::Flip];
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(cards.iter().all(|c| {
                c.data
                    .monster_subtypes
                    .as_ref()
                    .map(|v| v.contains(&ygo::MonsterSubtype::Flip))
                    .unwrap_or(false)
            }));
            assert!(!cards.iter().any(|c| c.id == c_bad.id));
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_multiple_subtypes() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let c_ok = ygo::NewCard {
                data: ygo::CardData {
                    name: "Tuner Flip Monster".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Effect),
                    monster_subtypes: Some(vec![
                        ygo::MonsterSubtype::Tuner,
                        ygo::MonsterSubtype::Flip,
                    ]),
                    ..Default::default()
                },
            };
            let c_bad = ygo::NewCard {
                data: ygo::CardData {
                    name: "Tuner Monster".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Effect),
                    monster_subtypes: Some(vec![ygo::MonsterSubtype::Tuner]),
                    ..Default::default()
                },
            };
            let c_ok = save_new(&client, &c_ok).await.unwrap();
            let c_bad = save_new(&client, &c_bad).await.unwrap();

            let mut filter = Filter::default();
            filter.subtype = vec![ygo::MonsterSubtype::Tuner, ygo::MonsterSubtype::Flip];
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(cards.iter().all(|c| {
                c.data
                    .monster_subtypes
                    .as_ref()
                    .map(|v| {
                        v.contains(&ygo::MonsterSubtype::Flip)
                            && v.contains(&ygo::MonsterSubtype::Tuner)
                    })
                    .unwrap_or(false)
            }));
            assert!(!cards.iter().any(|c| c.id == c_bad.id));
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_atk_min() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let low = ygo::NewCard {
                data: ygo::CardData {
                    name: "Low ATK".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_atk: Some(1000),
                    ..Default::default()
                },
            };
            let high = ygo::NewCard {
                data: ygo::CardData {
                    name: "High ATK".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_atk: Some(2500),
                    ..Default::default()
                },
            };
            let _ = save_new(&client, &low).await.unwrap();
            let c_ok = save_new(&client, &high).await.unwrap();

            let mut filter = Filter::default();
            filter.atk_min = Some(2000);
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(
                cards
                    .iter()
                    .all(|c| c.data.monster_atk.unwrap_or_default() >= 2000)
            );
            assert!(
                !cards
                    .iter()
                    .any(|c| c.data.monster_atk.unwrap_or_default() < 2000)
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_atk_max() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let low = ygo::NewCard {
                data: ygo::CardData {
                    name: "Low ATK".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_atk: Some(1200),
                    ..Default::default()
                },
            };
            let high = ygo::NewCard {
                data: ygo::CardData {
                    name: "High ATK".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_atk: Some(2800),
                    ..Default::default()
                },
            };
            let c_ok = save_new(&client, &low).await.unwrap();
            let _ = save_new(&client, &high).await.unwrap();

            let mut filter = Filter::default();
            filter.atk_max = Some(2000);
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(
                cards
                    .iter()
                    .all(|c| c.data.monster_atk.unwrap_or_default() <= 2000)
            );
            assert!(
                !cards
                    .iter()
                    .any(|c| c.data.monster_atk.unwrap_or_default() > 2000)
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_def_min() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let low = ygo::NewCard {
                data: ygo::CardData {
                    name: "Low DEF".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_def: Some(800),
                    ..Default::default()
                },
            };
            let high = ygo::NewCard {
                data: ygo::CardData {
                    name: "High DEF".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_def: Some(2500),
                    ..Default::default()
                },
            };
            let _ = save_new(&client, &low).await.unwrap();
            let c_ok = save_new(&client, &high).await.unwrap();

            let mut filter = Filter::default();
            filter.def_min = Some(2000);
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(
                cards
                    .iter()
                    .all(|c| c.data.monster_def.unwrap_or_default() >= 2000)
            );
            assert!(
                !cards
                    .iter()
                    .any(|c| c.data.monster_def.unwrap_or_default() < 2000)
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_def_max() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let low = ygo::NewCard {
                data: ygo::CardData {
                    name: "Low DEF".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_def: Some(1000),
                    ..Default::default()
                },
            };
            let high = ygo::NewCard {
                data: ygo::CardData {
                    name: "High DEF".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_def: Some(3000),
                    ..Default::default()
                },
            };
            let c_ok = save_new(&client, &low).await.unwrap();
            let _ = save_new(&client, &high).await.unwrap();

            let mut filter = Filter::default();
            filter.def_max = Some(2000);
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(
                cards
                    .iter()
                    .all(|c| c.data.monster_def.unwrap_or_default() <= 2000)
            );
            assert!(
                !cards
                    .iter()
                    .any(|c| c.data.monster_def.unwrap_or_default() > 2000)
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_level_min() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let low = ygo::NewCard {
                data: ygo::CardData {
                    name: "Low LV".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_level: Some(2),
                    ..Default::default()
                },
            };
            let high = ygo::NewCard {
                data: ygo::CardData {
                    name: "High LV".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_level: Some(7),
                    ..Default::default()
                },
            };
            let _ = save_new(&client, &low).await.unwrap();
            let c_ok = save_new(&client, &high).await.unwrap();

            let mut filter = Filter::default();
            filter.level_min = Some(5);
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(
                cards
                    .iter()
                    .all(|c| c.data.monster_level.unwrap_or_default() >= 5)
            );
            assert!(
                !cards
                    .iter()
                    .any(|c| c.data.monster_level.unwrap_or_default() < 5)
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_level_max() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let low = ygo::NewCard {
                data: ygo::CardData {
                    name: "Low LV".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_level: Some(3),
                    ..Default::default()
                },
            };
            let high = ygo::NewCard {
                data: ygo::CardData {
                    name: "High LV".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_level: Some(8),
                    ..Default::default()
                },
            };
            let c_ok = save_new(&client, &low).await.unwrap();
            let _ = save_new(&client, &high).await.unwrap();

            let mut filter = Filter::default();
            filter.level_max = Some(4);
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(
                cards
                    .iter()
                    .all(|c| c.data.monster_level.unwrap_or_default() <= 4)
            );
            assert!(
                !cards
                    .iter()
                    .any(|c| c.data.monster_level.unwrap_or_default() > 4)
            );
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_spell_kind() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let c_ok = ygo::NewCard {
                data: ygo::CardData {
                    name: "Quick-Play".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Spell,
                    spell_kind: Some(ygo::SpellKind::QuickPlay),
                    ..Default::default()
                },
            };
            let c_bad = ygo::NewCard {
                data: ygo::CardData {
                    name: "Field Spell".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Spell,
                    spell_kind: Some(ygo::SpellKind::Field),
                    ..Default::default()
                },
            };
            let c_ok = save_new(&client, &c_ok).await.unwrap();
            let c_bad = save_new(&client, &c_bad).await.unwrap();

            let mut filter = Filter::default();
            filter.spell = vec![ygo::SpellKind::QuickPlay];
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(
                cards
                    .iter()
                    .all(|c| c.data.spell_kind == Some(ygo::SpellKind::QuickPlay))
            );
            assert!(!cards.iter().any(|c| c.id == c_bad.id));
        })
        .await;
    }

    #[tokio::test]
    async fn test_filter_by_trap_kind() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let c_ok = ygo::NewCard {
                data: ygo::CardData {
                    name: "Normal Trap".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Trap,
                    trap_kind: Some(ygo::TrapKind::Normal),
                    ..Default::default()
                },
            };
            let c_bad = ygo::NewCard {
                data: ygo::CardData {
                    name: "Counter Trap".into(),
                    description: "".into(),
                    kind: ygo::CardKind::Trap,
                    trap_kind: Some(ygo::TrapKind::Counter),
                    ..Default::default()
                },
            };
            let c_ok = save_new(&client, &c_ok).await.unwrap();
            let c_bad = save_new(&client, &c_bad).await.unwrap();

            let mut filter = Filter::default();
            filter.trap = vec![ygo::TrapKind::Normal];
            let (cards, _) = get_page(&client, Some(filter), None, 50, None)
                .await
                .unwrap();
            assert!(cards.iter().any(|c| c.id == c_ok.id));
            assert!(
                cards
                    .iter()
                    .all(|c| c.data.trap_kind == Some(ygo::TrapKind::Normal))
            );
            assert!(!cards.iter().any(|c| c.id == c_bad.id));
        })
        .await;
    }

    #[tokio::test]
    async fn test_sort_by_name_asc() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            // Unordered: Charlie, Alice, Bob
            let a = ygo::NewCard {
                data: ygo::CardData {
                    name: "Charlie".into(),
                    kind: ygo::CardKind::Spell,
                    ..Default::default()
                },
            };
            let b = ygo::NewCard {
                data: ygo::CardData {
                    name: "Alice".into(),
                    kind: ygo::CardKind::Spell,
                    ..Default::default()
                },
            };
            let c = ygo::NewCard {
                data: ygo::CardData {
                    name: "Bob".into(),
                    kind: ygo::CardKind::Spell,
                    ..Default::default()
                },
            };

            let _ = save_new(&client, &a).await.unwrap();
            let _ = save_new(&client, &b).await.unwrap();
            let _ = save_new(&client, &c).await.unwrap();

            let filter = Filter::default();
            let sort = Sort {
                sort: SortingField::Name,
                dir: SortingDirection::Asc,
            };

            let (cards, next) = get_page(&client, Some(filter), Some(sort), 10, None)
                .await
                .unwrap();

            assert_eq!(cards.len(), 3);
            assert!(next.is_none());
            let names: Vec<_> = cards.iter().map(|c| c.data.name.as_str()).collect();
            assert_eq!(names, vec!["Alice", "Bob", "Charlie"]);
        })
        .await;
    }

    #[tokio::test]
    async fn test_sort_by_atk_asc() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let a = ygo::NewCard {
                data: ygo::CardData {
                    name: "A".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_atk: Some(300),
                    ..Default::default()
                },
            };
            let b = ygo::NewCard {
                data: ygo::CardData {
                    name: "B".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_atk: Some(100),
                    ..Default::default()
                },
            };
            let c = ygo::NewCard {
                data: ygo::CardData {
                    name: "C".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_atk: Some(200),
                    ..Default::default()
                },
            };

            let _ = save_new(&client, &a).await.unwrap();
            let _ = save_new(&client, &b).await.unwrap();
            let _ = save_new(&client, &c).await.unwrap();

            let filter = Filter::default();
            let sort = Sort {
                sort: SortingField::Atk,
                dir: SortingDirection::Asc,
            };

            let (cards, _) = get_page(&client, Some(filter), Some(sort), 10, None)
                .await
                .unwrap();

            let atks: Vec<i16> = cards.iter().map(|c| c.data.monster_atk.unwrap()).collect();
            assert_eq!(atks, vec![100, 200, 300]);
        })
        .await;
    }

    #[tokio::test]
    async fn test_sort_by_def_asc() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let a = ygo::NewCard {
                data: ygo::CardData {
                    name: "A".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_def: Some(900),
                    ..Default::default()
                },
            };
            let b = ygo::NewCard {
                data: ygo::CardData {
                    name: "B".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_def: Some(100),
                    ..Default::default()
                },
            };
            let c = ygo::NewCard {
                data: ygo::CardData {
                    name: "C".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_def: Some(500),
                    ..Default::default()
                },
            };

            let _ = save_new(&client, &a).await.unwrap();
            let _ = save_new(&client, &b).await.unwrap();
            let _ = save_new(&client, &c).await.unwrap();

            let filter = Filter::default();
            let sort = Sort {
                sort: SortingField::Def,
                dir: SortingDirection::Asc,
            };

            let (cards, _) = get_page(&client, Some(filter), Some(sort), 10, None)
                .await
                .unwrap();

            let defs: Vec<i16> = cards.iter().map(|c| c.data.monster_def.unwrap()).collect();
            assert_eq!(defs, vec![100, 500, 900]);
        })
        .await;
    }

    #[tokio::test]
    async fn test_sort_by_level_asc() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let a = ygo::NewCard {
                data: ygo::CardData {
                    name: "A".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_level: Some(6),
                    ..Default::default()
                },
            };
            let b = ygo::NewCard {
                data: ygo::CardData {
                    name: "B".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_level: Some(2),
                    ..Default::default()
                },
            };
            let c = ygo::NewCard {
                data: ygo::CardData {
                    name: "C".into(),
                    kind: ygo::CardKind::Monster,
                    monster_kind: Some(ygo::MonsterKind::Normal),
                    monster_level: Some(4),
                    ..Default::default()
                },
            };

            let _ = save_new(&client, &a).await.unwrap();
            let _ = save_new(&client, &b).await.unwrap();
            let _ = save_new(&client, &c).await.unwrap();

            let filter = Filter::default();
            let sort = Sort {
                sort: SortingField::Level,
                dir: SortingDirection::Asc,
            };

            let (cards, _) = get_page(&client, Some(filter), Some(sort), 10, None)
                .await
                .unwrap();

            let levels: Vec<i16> = cards
                .iter()
                .map(|c| c.data.monster_level.unwrap())
                .collect();
            assert_eq!(levels, vec![2, 4, 6]);
        })
        .await;
    }

    #[tokio::test]
    async fn test_sort_by_tcg_date_asc() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let d1 = chrono::NaiveDate::from_ymd_opt(2020, 5, 1).unwrap();
            let d2 = chrono::NaiveDate::from_ymd_opt(2018, 1, 10).unwrap();
            let d3 = chrono::NaiveDate::from_ymd_opt(2019, 12, 15).unwrap();

            let a = ygo::NewCard {
                data: ygo::CardData {
                    name: "A".into(),
                    kind: ygo::CardKind::Monster,
                    tcg_date: Some(d1),
                    ..Default::default()
                },
            };
            let b = ygo::NewCard {
                data: ygo::CardData {
                    name: "B".into(),
                    kind: ygo::CardKind::Monster,
                    tcg_date: Some(d2),
                    ..Default::default()
                },
            };
            let c = ygo::NewCard {
                data: ygo::CardData {
                    name: "C".into(),
                    kind: ygo::CardKind::Monster,
                    tcg_date: Some(d3),
                    ..Default::default()
                },
            };

            let _ = save_new(&client, &a).await.unwrap();
            let _ = save_new(&client, &b).await.unwrap();
            let _ = save_new(&client, &c).await.unwrap();

            let filter = Filter::default();
            let sort = Sort {
                sort: SortingField::TcgDate,
                dir: SortingDirection::Asc,
            };

            let (cards, _) = get_page(&client, Some(filter), Some(sort), 10, None)
                .await
                .unwrap();

            let dates: Vec<chrono::NaiveDate> =
                cards.iter().map(|c| c.data.tcg_date.unwrap()).collect();
            assert_eq!(dates, vec![d2, d3, d1]);
        })
        .await;
    }

    #[tokio::test]
    async fn test_sort_by_ocg_date_asc() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            let d1 = chrono::NaiveDate::from_ymd_opt(2021, 7, 20).unwrap();
            let d2 = chrono::NaiveDate::from_ymd_opt(2020, 3, 5).unwrap();
            let d3 = chrono::NaiveDate::from_ymd_opt(2020, 11, 30).unwrap();

            let a = ygo::NewCard {
                data: ygo::CardData {
                    name: "A".into(),
                    kind: ygo::CardKind::Monster,
                    ocg_date: Some(d1),
                    ..Default::default()
                },
            };
            let b = ygo::NewCard {
                data: ygo::CardData {
                    name: "B".into(),
                    kind: ygo::CardKind::Monster,
                    ocg_date: Some(d2),
                    ..Default::default()
                },
            };
            let c = ygo::NewCard {
                data: ygo::CardData {
                    name: "C".into(),
                    kind: ygo::CardKind::Monster,
                    ocg_date: Some(d3),
                    ..Default::default()
                },
            };

            let _ = save_new(&client, &a).await.unwrap();
            let _ = save_new(&client, &b).await.unwrap();
            let _ = save_new(&client, &c).await.unwrap();

            let filter = Filter::default();
            let sort = Sort {
                sort: SortingField::OcgDate,
                dir: SortingDirection::Asc,
            };

            let (cards, _) = get_page(&client, Some(filter), Some(sort), 10, None)
                .await
                .unwrap();

            let dates: Vec<chrono::NaiveDate> =
                cards.iter().map(|c| c.data.ocg_date.unwrap()).collect();
            assert_eq!(dates, vec![d2, d3, d1]);
        })
        .await;
    }

    #[tokio::test]
    async fn test_pagination_cursor_three_items_two_per_page() {
        with_db_pool(async move |db| {
            let client = db.get().await.expect("db");

            // Insert 3 items, unordered by name: Charlie, Alice, Bob
            let a = ygo::NewCard {
                data: ygo::CardData {
                    name: "Charlie".into(),
                    kind: ygo::CardKind::Spell,
                    ..Default::default()
                },
            };
            let b = ygo::NewCard {
                data: ygo::CardData {
                    name: "Alice".into(),
                    kind: ygo::CardKind::Spell,
                    ..Default::default()
                },
            };
            let c = ygo::NewCard {
                data: ygo::CardData {
                    name: "Bob".into(),
                    kind: ygo::CardKind::Spell,
                    ..Default::default()
                },
            };

            let _ = save_new(&client, &a).await.unwrap();
            let _ = save_new(&client, &b).await.unwrap();
            let _ = save_new(&client, &c).await.unwrap();

            // Page 1: limit 2, sorted by name ASC -> [Alice, Bob]
            let (p1, next) = get_page(
                &client,
                None,
                Some(Sort {
                    sort: SortingField::Name,
                    dir: SortingDirection::Asc,
                }),
                2,
                None,
            )
            .await
            .unwrap();
            assert_eq!(p1.len(), 2);
            let names1: Vec<_> = p1.iter().map(|c| c.data.name.as_str()).collect();
            assert_eq!(names1, vec!["Alice", "Bob"]);
            assert!(next.is_some());

            // Page 2 using cursor -> [Charlie]
            let (p2, next2) = get_page(
                &client,
                None,
                Some(Sort {
                    sort: SortingField::Name,
                    dir: SortingDirection::Asc,
                }),
                2,
                next,
            )
            .await
            .unwrap();
            assert_eq!(p2.len(), 1);
            assert_eq!(p2[0].data.name, "Charlie");
            assert!(next2.is_none());
        })
        .await;
    }
}
