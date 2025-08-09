use tokio_postgres::{Client, Row};

use crate::{database::TimestampWithTimeZone, models::ygo, prelude::*};

pub async fn get_one(client: &Client, id: i32) -> Result<ygo::Card> {
    let query = "SELECT * FROM ygo_cards WHERE id = $1";
    let row = &client
        .query_opt(query, &[&id])
        .await?
        .ok_or_else(|| AppError::NotFound {
            resource: id.into(),
        })?;

    let card = row.try_into()?;
    Ok(card)
}

pub async fn get_all(client: &Client) -> Result<Vec<ygo::Card>> {
    let query = "SELECT * FROM ygo_cards";
    let rows = client.query(query, &[]).await?;

    let cards = rows
        .iter()
        .map(|row| row.try_into())
        .collect::<Result<Vec<ygo::Card>>>()?;

    Ok(cards)
}

/// Insert a new card and return the created record.
pub async fn save_new(client: &Client, new_card: &ygo::NewCard) -> Result<ygo::Card> {
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
                tcgplayer_price,
                cardmarket_price,
                ebay_price,
                coolstuffinc_price,
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
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24
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
                &card_data.tcgplayer_price,
                &card_data.cardmarket_price,
                &card_data.ebay_price,
                &card_data.coolstuffinc_price,
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
pub async fn save(client: &Client, card: &ygo::Card) -> Result<ygo::Card> {
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
                tcgplayer_price = $9,
                cardmarket_price = $10,
                ebay_price = $11,
                coolstuffinc_price = $12,
                monster_kind = $13,
                monster_attribute = $14,
                monster_race = $15,
                monster_subtypes = $16,
                monster_atk = $17,
                monster_def = $18,
                monster_level = $19,
                monster_pendulum_scale = $20,
                monster_pendulum_effect = $21,
                monster_link_arrows = $22,
                spell_kind = $23,
                trap_kind = $24,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $25
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
                &d.tcgplayer_price,
                &d.cardmarket_price,
                &d.ebay_price,
                &d.coolstuffinc_price,
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

    let row = row.ok_or_else(|| AppError::NotFound {
        resource: id.into(),
    })?;
    let updated: ygo::Card = (&row).try_into()?;
    Ok(updated)
}

/// Seeds the database with a fixed set of sample Yu-Gi-Oh! cards.
/// Used by the import HTTP handler and tests.
pub async fn import_sample_cards(client: &Client) -> Result<Vec<ygo::Card>> {
    client
        .execute(
            r#"
                INSERT INTO ygo_cards (
                    id,
                    name,
                    description,
                    kind,
                    monster_kind,
                    monster_attribute,
                    monster_race,
                    monster_level,
                    monster_atk,
                    monster_def
                ) VALUES (
                    1,
                    'Blue-eyes White Dragon',
                    'This legendary dragon is a powerful engine of destruction. Virtually invincible, very few have faced this awesome creature and lived to tell the tale.',
                    'monster',
                    'normal',
                    'light',
                    'dragon',
                    8,
                    3000,
                    2500
                ), (
                    2,
                    'Dark Magician',
                    'The ultimate wizard in terms of attack and defense.',
                    'monster',
                    'normal',
                    'dark',
                    'spellcaster',
                    7,
                    2500,
                    2100
                ) ON CONFLICT DO NOTHING;
            "#,
            &[],
        )
        .await?;

    let rows = client
        .query(
            "SELECT * FROM ygo_cards WHERE name = ANY($1)",
            &[&vec!["Blue-eyes White Dragon", "Dark Magician"]],
        )
        .await?;

    let cards = rows
        .iter()
        .map(|r| r.try_into())
        .collect::<Result<Vec<ygo::Card>>>()?;

    Ok(cards)
}

impl TryFrom<&Row> for ygo::Card {
    type Error = AppError;

    /// Converts a database row into a YugiohCard struct
    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let id: i32 = value.get("id");
        let updated_at: TimestampWithTimeZone = value.get("updated_at");

        Ok(Self {
            id,
            updated_at: updated_at.0,
            data: value.try_into()?,
        })
    }
}

impl TryFrom<&Row> for ygo::CardData {
    type Error = AppError;

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
            tcgplayer_price: value.try_get("tcgplayer_price")?,
            cardmarket_price: value.try_get("cardmarket_price")?,
            ebay_price: value.try_get("ebay_price")?,
            coolstuffinc_price: value.try_get("coolstuffinc_price")?,
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

            let fetched = get_one(&client, created.id).await.expect("fetch");
            assert_eq!(fetched, created);
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
            assert_eq!(updated.id, created.id);
            assert_eq!(updated.data.name, "Updated Name");
            assert_eq!(updated.data.description, "After");
            assert_eq!(updated.data.monster_atk, Some(1600));

            // Fetch to confirm persistence
            let fetched = get_one(&client, created.id).await.expect("fetch");
            assert_eq!(fetched.data.name, "Updated Name");
            assert_eq!(fetched.data.description, "After");
            assert_eq!(fetched.data.monster_atk, Some(1600));
        })
        .await;
    }
}
