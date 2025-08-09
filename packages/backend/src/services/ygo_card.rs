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
    fn try_from(value: &Row) -> Result<Self, self::AppError> {
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
