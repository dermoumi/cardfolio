use anyhow::Context;
use serde::Deserialize;
use tokio_postgres::Client;

use crate::models::ygo::{Card, CardData};
use crate::{models::ygo, services::ygo as service};

#[derive(Debug, Deserialize)]
struct YgoProDeckList {
    data: Vec<YgoProDeckCard>,
}

#[derive(Debug, Deserialize)]
struct YgoProDeckCard {
    id: i64,
    name: String,
    #[serde(rename = "frameType")]
    frame_type: String,
    desc: String,
    // For monsters, this is monster race (e.g., Dragon). For spells/traps, this is their subtype label.
    race: Option<String>,
    // Monster-only fields
    #[serde(rename = "typeline")]
    type_line: Option<Vec<String>>, // e.g., ["Dragon", "Effect", "Flip"], or None for spells/traps
    attribute: Option<String>,
    level: Option<i32>,
    atk: Option<i32>,
    def: Option<i32>,
    #[serde(rename = "linkmarkers")]
    link_markers: Option<Vec<String>>,
    #[serde(rename = "linkval")]
    link_val: Option<i32>,
    #[serde(rename = "scale")]
    pendulum_scale: Option<i32>,
    #[serde(rename = "pend_desc")]
    pendulum_desc: Option<String>,
    monster_desc: Option<String>, // Monster description for Pendulum cards
    misc_info: Option<(YgoProDeckMiscInfo,)>, // tcg/ocg dates, konami_id, etc
}

#[derive(Debug, Deserialize)]
struct YgoProDeckMiscInfo {
    tcg_date: Option<String>,
    ocg_date: Option<String>,
    konami_id: Option<i32>,
}

impl YgoProDeckCard {
    fn get_monster_attribute(&self) -> Option<ygo::MonsterAttribute> {
        if !self.is_monster() {
            return None; // Not a monster card
        }

        self.attribute
            .as_ref()
            .map(|attr| match attr.to_lowercase().as_str() {
                "dark" => ygo::MonsterAttribute::Dark,
                "divine" => ygo::MonsterAttribute::Divine,
                "earth" => ygo::MonsterAttribute::Earth,
                "fire" => ygo::MonsterAttribute::Fire,
                "light" => ygo::MonsterAttribute::Light,
                "water" => ygo::MonsterAttribute::Water,
                "wind" => ygo::MonsterAttribute::Wind,
                _ => ygo::MonsterAttribute::Other,
            })
    }

    fn get_monster_race(&self) -> Option<ygo::MonsterRace> {
        if !self.is_monster() {
            return None; // Not a monster card
        }

        self.race
            .as_ref()
            .map(|race| match race.to_lowercase().as_str() {
                "aqua" => ygo::MonsterRace::Aqua,
                "beast" => ygo::MonsterRace::Beast,
                "beast-warrior" => ygo::MonsterRace::BeastWarrior,
                "creator-god" => ygo::MonsterRace::CreatorGod,
                "cyberse" => ygo::MonsterRace::Cyberse,
                "dinosaur" => ygo::MonsterRace::Dinosaur,
                "divine-beast" => ygo::MonsterRace::DivineBeast,
                "dragon" => ygo::MonsterRace::Dragon,
                "fairy" => ygo::MonsterRace::Fairy,
                "fiend" => ygo::MonsterRace::Fiend,
                "fish" => ygo::MonsterRace::Fish,
                "illusion" => ygo::MonsterRace::Illusion,
                "insect" => ygo::MonsterRace::Insect,
                "machine" => ygo::MonsterRace::Machine,
                "plant" => ygo::MonsterRace::Plant,
                "psychic" => ygo::MonsterRace::Psychic,
                "pyro" => ygo::MonsterRace::Pyro,
                "reptile" => ygo::MonsterRace::Reptile,
                "rock" => ygo::MonsterRace::Rock,
                "sea serpent" => ygo::MonsterRace::SeaSerpent,
                "spellcaster" => ygo::MonsterRace::Spellcaster,
                "thunder" => ygo::MonsterRace::Thunder,
                "warrior" => ygo::MonsterRace::Warrior,
                "winged beast" => ygo::MonsterRace::WingedBeast,
                "wyrm" => ygo::MonsterRace::Wyrm,
                "zombie" => ygo::MonsterRace::Zombie,
                _ => ygo::MonsterRace::Other, // Default case
            })
    }

    fn get_link_arrows(&self) -> Option<ygo::LinkArrows> {
        if self.frame_type != "link" {
            return None; // Not a link monster
        }

        self.link_markers.as_ref().map(|labels| {
            use ygo::LinkArrows as L;

            labels.iter().fold(L::empty(), |mut acc, s| {
                match s.to_lowercase().as_str() {
                    "top-left" => acc |= L::TopLeft,
                    "top" => acc |= L::Top,
                    "top-right" => acc |= L::TopRight,
                    "left" => acc |= L::Left,
                    "right" => acc |= L::Right,
                    "bottom-left" => acc |= L::BottomLeft,
                    "bottom" => acc |= L::Bottom,
                    "bottom-right" => acc |= L::BottomRight,
                    _ => {}
                }
                acc
            })
        })
    }

    fn get_spell_kind(&self) -> Option<ygo::SpellKind> {
        if self.frame_type != "spell" {
            return None; // Not a spell card
        }

        self.race
            .as_ref()
            .and_then(|race| match race.to_lowercase().as_str() {
                "normal" => Some(ygo::SpellKind::Normal),
                "quick-play" => Some(ygo::SpellKind::QuickPlay),
                "equip" => Some(ygo::SpellKind::Equip),
                "field" => Some(ygo::SpellKind::Field),
                "continuous" => Some(ygo::SpellKind::Continuous),
                "ritual" => Some(ygo::SpellKind::Ritual),
                _ => None,
            })
    }

    fn get_trap_kind(&self) -> Option<ygo::TrapKind> {
        if self.frame_type != "trap" {
            return None; // Not a trap card
        }

        self.race
            .as_ref()
            .and_then(|race| match race.to_lowercase().as_str() {
                "normal" => Some(ygo::TrapKind::Normal),
                "counter" => Some(ygo::TrapKind::Counter),
                "continuous" => Some(ygo::TrapKind::Continuous),
                _ => None,
            })
    }

    fn get_monster_subtypes(&self) -> (Option<ygo::MonsterKind>, Option<Vec<ygo::MonsterSubtype>>) {
        let type_lines = match &self.type_line {
            Some(lines) if self.is_monster() => lines,
            _ => return (None, None),
        };

        let mut monster_kind = None;
        let mut monster_subtypes = Vec::new();

        for type_name in type_lines.iter() {
            let type_name_lower = type_name.to_lowercase();

            match type_name_lower.as_str() {
                "effect" => {
                    if monster_kind.is_none() {
                        monster_kind = Some(ygo::MonsterKind::Effect)
                    }
                }
                "normal" => monster_kind = Some(ygo::MonsterKind::Normal),
                "token" => monster_kind = Some(ygo::MonsterKind::Token),
                "fusion" => monster_kind = Some(ygo::MonsterKind::Fusion),
                "ritual" => monster_kind = Some(ygo::MonsterKind::Ritual),
                "synchro" => monster_kind = Some(ygo::MonsterKind::Synchro),
                "xyz" => monster_kind = Some(ygo::MonsterKind::Xyz),
                "link" => monster_kind = Some(ygo::MonsterKind::Link),
                "gemini" => monster_subtypes.push(ygo::MonsterSubtype::Gemini),
                "flip" => monster_subtypes.push(ygo::MonsterSubtype::Flip),
                "spirit" => monster_subtypes.push(ygo::MonsterSubtype::Spirit),
                "toon" => monster_subtypes.push(ygo::MonsterSubtype::Toon),
                "tuner" => monster_subtypes.push(ygo::MonsterSubtype::Tuner),
                "union" => monster_subtypes.push(ygo::MonsterSubtype::Union),
                _ => {}
            }
        }

        let monster_subtypes = match monster_subtypes.is_empty() {
            true => None,
            false => Some(monster_subtypes),
        };

        (monster_kind, monster_subtypes)
    }

    fn get_monster_atk(&self) -> Option<i16> {
        if !self.is_monster() {
            return None; // Not a monster card
        }

        Self::to_i16(self.atk)
    }

    fn get_monster_def(&self) -> Option<i16> {
        if !self.is_monster() {
            return None; // Not a monster card
        }

        Self::to_i16(self.def)
    }

    fn get_monster_level(&self) -> Option<i16> {
        if !self.is_monster() {
            return None; // Not a monster card
        }

        match self.frame_type.as_str() {
            "link" => Self::to_i16(self.link_val), // Link monsters use link_val for level
            _ => Self::to_i16(self.level),
        }
    }

    fn get_monster_pendulum_scale(&self) -> Option<i16> {
        if !self.is_monster() {
            return None; // Not a monster card
        }

        Self::to_i16(self.pendulum_scale)
    }

    fn get_monster_pendulum_effect(&self) -> Option<String> {
        if !self.is_monster() {
            return None; // Not a monster card
        }

        self.pendulum_desc.clone()
    }

    fn is_monster(&self) -> bool {
        match self.frame_type.as_str() {
            "spell" | "trap" => false, // Not a monster card
            _ => true,
        }
    }

    fn to_i16(opt: Option<i32>) -> Option<i16> {
        opt.map(|v| v.clamp(i16::MIN as i32, i16::MAX as i32) as i16)
    }

    fn parse_date_opt(s: &Option<String>) -> Option<chrono::NaiveDate> {
        s.as_ref()
            .and_then(|v| chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d").ok())
    }

    fn get_kind(&self) -> ygo::CardKind {
        match self.frame_type.as_str() {
            "spell" => ygo::CardKind::Spell,
            "trap" => ygo::CardKind::Trap,
            _ => ygo::CardKind::Monster,
        }
    }

    fn get_description(&self) -> &str {
        if let Some(desc) = &self.monster_desc {
            desc
        } else {
            &self.desc
        }
    }

    fn get_password(&self) -> Option<String> {
        // Only IDs of 8 digits are valid passwords
        if self.id <= 99_999_999 {
            Some(self.id.to_string())
        } else {
            None
        }
    }
}

impl TryFrom<YgoProDeckCard> for CardData {
    type Error = anyhow::Error;

    fn try_from(card: YgoProDeckCard) -> Result<Self, Self::Error> {
        let name = card.name.clone();
        let description = card.get_description().to_string();
        let kind = card.get_kind();
        let password = card.get_password();
        let konami_id = card.misc_info.as_ref().and_then(|info| info.0.konami_id);
        let treated_as = None;

        let tcg_date = card
            .misc_info
            .as_ref()
            .and_then(|info| YgoProDeckCard::parse_date_opt(&info.0.tcg_date));
        let ocg_date = card
            .misc_info
            .as_ref()
            .and_then(|info| YgoProDeckCard::parse_date_opt(&info.0.ocg_date));

        // Monster-specific fields
        let (monster_kind, monster_subtypes) = card.get_monster_subtypes();
        let monster_attribute = card.get_monster_attribute();
        let monster_race = card.get_monster_race();
        let monster_atk = card.get_monster_atk();
        let monster_def = card.get_monster_def();
        let monster_level = card.get_monster_level();
        let monster_pendulum_scale = card.get_monster_pendulum_scale();
        let monster_pendulum_effect = card.get_monster_pendulum_effect();
        let monster_link_arrows = card.get_link_arrows();

        // Spell specific data
        let spell_kind = card.get_spell_kind();

        // Trap specific data
        let trap_kind = card.get_trap_kind();

        // Build card data
        let card = CardData {
            name,
            description,
            kind,
            password,
            konami_id,
            treated_as, // TODO
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
        };

        Ok(card)
    }
}

/// Parses a json list
async fn parse_json_list(json: &str) -> anyhow::Result<impl Iterator<Item = YgoProDeckCard>> {
    let parsed_list: YgoProDeckList =
        serde_json::from_str(json).with_context(|| "Failed to parse ygoprodeck JSON deck file")?;

    Ok(parsed_list.data.into_iter())
}

/// Utility to get the existing card from the database
async fn get_existing_card(client: &Client, card_data: &CardData) -> anyhow::Result<Option<Card>> {
    // Check by Konami ID
    if let Some(konami_id) = card_data.konami_id {
        return Ok(service::card::get_by_konami_id(client, konami_id).await?);
    }

    if let Some(password) = &card_data.password {
        return Ok(service::card::get_by_password(client, password).await?);
    }

    anyhow::bail!(
        "Card '{}' has no Konami ID or password, cannot check for existing card",
        card_data.name
    );
}

/// Imports cards from a json string
async fn import_from_json_str(client: &Client, json: &str) -> anyhow::Result<(usize, usize)> {
    let mut inserted = 0;
    let mut updated = 0;

    for card in parse_json_list(json).await? {
        let card_data: CardData = card.try_into()?;
        let existing_card = match get_existing_card(client, &card_data).await {
            Ok(card) => card,
            Err(err) => {
                tracing::warn!("{}. Skipping...", err);
                continue;
            }
        };

        if let Some(mut card) = existing_card {
            card.data = card_data.clone();
            service::card::save(client, &card).await?;
            updated += 1;
        } else {
            let new_card = ygo::NewCard {
                data: card_data.clone(),
            };
            service::card::save_new(client, &new_card).await?;
            inserted += 1;
        }
    }

    Ok((inserted, updated))
}

/// Imports YgoProDeck cards into the database
pub async fn import(client: &Client) -> anyhow::Result<(usize, usize)> {
    const ENDPOINT: &str = "https://db.ygoprodeck.com/api/v7/cardinfo.php?misc=yes&sort=new";

    let json = reqwest::get(ENDPOINT).await?.text().await?;

    import_from_json_str(client, &json).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::ygo, test_utils::with_db_pool};

    async fn json_to_card_data(json: &str) -> anyhow::Result<Vec<CardData>> {
        parse_json_list(json)
            .await?
            .map(|card| card.try_into())
            .collect::<anyhow::Result<Vec<CardData>>>()
    }

    #[tokio::test]
    async fn test_normal_monster() {
        let json = r#"{"data":[{
            "id": 89943723,
            "name": "Elemental HERO Neos",
            "typeline": ["Warrior", "Normal"],
            "type": "Normal Monster",
            "humanReadableCardType": "Normal Monster",
            "frameType": "normal",
            "desc": "''A new Elemental HERO has arrived from Neo-Space! When he initiates a Contact Fusion with a Neo-Spacian his unknown powers are unleashed.''",
            "race": "Warrior",
            "atk": 2500,
            "def": 2000,
            "level": 7,
            "attribute": "LIGHT",
            "archetype": "Elemental HERO",
            "ygoprodeck_url": "https://ygoprodeck.com/card/elemental-hero-neos-7516",
            "card_sets": [
                {
                    "set_name": "Battles of Legend: Monster Mayhem",
                    "set_code": "BLMM-EN003",
                    "set_rarity": "Secret Rare",
                    "set_rarity_code": "(ScR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Battles of Legend: Monster Mayhem",
                    "set_code": "BLMM-EN003",
                    "set_rarity": "Starlight Rare",
                    "set_rarity_code": "(StR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Collectible Tins 2006 Wave 1",
                    "set_code": "CT03-EN001",
                    "set_rarity": "Secret Rare",
                    "set_rarity_code": "(ScR)",
                    "set_price": "22.32"
                },
                {
                    "set_name": "Duel Power",
                    "set_code": "DUPO-EN102",
                    "set_rarity": "Ultra Rare",
                    "set_rarity_code": "(UR)",
                    "set_price": "2.6"
                },
                {
                    "set_name": "Duel Terminal - Preview Wave 1",
                    "set_code": "DTP1-EN005",
                    "set_rarity": "Duel Terminal Normal Parallel Rare",
                    "set_rarity_code": "(DNPR)",
                    "set_price": "0.00"
                },
                {
                    "set_name": "Duel Terminal - Preview Wave 2",
                    "set_code": "DTP1-EN005",
                    "set_rarity": "Duel Terminal Rare Parallel Rare",
                    "set_rarity_code": "(DRPR)",
                    "set_price": "279.99"
                },
                {
                    "set_name": "Duel Terminal 1",
                    "set_code": "DT01-EN005",
                    "set_rarity": "Duel Terminal Rare Parallel Rare",
                    "set_rarity_code": "(DRPR)",
                    "set_price": "17.34"
                },
                {
                    "set_name": "Duelist League 3 participation cards",
                    "set_code": "DL12-EN001",
                    "set_rarity": "Rare",
                    "set_rarity_code": "(R)",
                    "set_price": "6.3"
                },
                {
                    "set_name": "Duelist Pack: Jaden Yuki 2",
                    "set_code": "DP03-EN001",
                    "set_rarity": "Common",
                    "set_rarity_code": "(C)",
                    "set_price": "1.75"
                },
                {
                    "set_name": "HERO Strike Structure Deck",
                    "set_code": "SDHS-EN007",
                    "set_rarity": "Common",
                    "set_rarity_code": "(C)",
                    "set_price": "1.27"
                },
                {
                    "set_name": "Legendary Collection 2: The Duel Academy Years Mega Pack",
                    "set_code": "LCGX-EN008",
                    "set_rarity": "Common",
                    "set_rarity_code": "(C)",
                    "set_price": "1.43"
                },
                {
                    "set_name": "Maximum Gold: El Dorado",
                    "set_code": "MGED-EN004",
                    "set_rarity": "Premium Gold Rare",
                    "set_rarity_code": "(PG)",
                    "set_price": "1.74"
                },
                {
                    "set_name": "Power of the Duelist",
                    "set_code": "POTD-EN001",
                    "set_rarity": "Common",
                    "set_rarity_code": "(C)",
                    "set_price": "1.47"
                },
                {
                    "set_name": "Ra Yellow Mega Pack",
                    "set_code": "RYMP-EN004",
                    "set_rarity": "Common",
                    "set_rarity_code": "(C)",
                    "set_price": "1.39"
                },
                {
                    "set_name": "Shadows in Valhalla",
                    "set_code": "SHVA-EN031",
                    "set_rarity": "Super Rare",
                    "set_rarity_code": "(SR)",
                    "set_price": "2.89"
                }
            ],
            "card_images": [
                {
                    "id": 89943723,
                    "image_url": "https://images.ygoprodeck.com/images/cards/89943723.jpg",
                    "image_url_small": "https://images.ygoprodeck.com/images/cards_small/89943723.jpg",
                    "image_url_cropped": "https://images.ygoprodeck.com/images/cards_cropped/89943723.jpg"
                },
                {
                    "id": 89943724,
                    "image_url": "https://images.ygoprodeck.com/images/cards/89943724.jpg",
                    "image_url_small": "https://images.ygoprodeck.com/images/cards_small/89943724.jpg",
                    "image_url_cropped": "https://images.ygoprodeck.com/images/cards_cropped/89943724.jpg"
                }
            ],
            "card_prices": [
                {
                    "cardmarket_price": "0.18",
                    "tcgplayer_price": "0.20",
                    "ebay_price": "2.99",
                    "amazon_price": "0.93",
                    "coolstuffinc_price": "0.25"
                }
            ],
            "misc_info": [
                {
                    "views": 523010,
                    "viewsweek": 909,
                    "upvotes": 117,
                    "downvotes": 15,
                    "formats": [
                        "Duel Links",
                        "Common Charity",
                        "Edison",
                        "Speed Duel",
                        "TCG",
                        "OCG",
                        "Master Duel"
                    ],
                    "treated_as": "Elemental HERO Neos",
                    "tcg_date": "2006-08-16",
                    "ocg_date": "2005-12-12",
                    "konami_id": 6653,
                    "has_effect": 0,
                    "md_rarity": "Ultra Rare"
                }
            ]
        }]}"#;

        let cards = json_to_card_data(json).await.expect("import");
        assert_eq!(cards.len(), 1);

        let card = &cards[0];
        assert_eq!(card.name, "Elemental HERO Neos");
        assert_eq!(
            card.description,
            "''A new Elemental HERO has arrived from Neo-Space! When he initiates a Contact Fusion with a Neo-Spacian his unknown powers are unleashed.''"
        );
        assert_eq!(card.password.as_deref(), Some("89943723"));
        assert_eq!(card.kind, ygo::CardKind::Monster);
        assert_eq!(card.monster_kind, Some(ygo::MonsterKind::Normal));
        assert_eq!(card.monster_attribute, Some(ygo::MonsterAttribute::Light));
        assert_eq!(card.monster_race, Some(ygo::MonsterRace::Warrior));
        assert_eq!(card.monster_level, Some(7));
        assert_eq!(card.monster_atk, Some(2500));
        assert_eq!(card.monster_def, Some(2000));
        assert_eq!(card.monster_pendulum_scale, None);
        assert_eq!(card.monster_pendulum_effect.as_deref(), None);
        assert_eq!(card.monster_link_arrows, None);
    }

    #[tokio::test]
    async fn test_effect_monster() {
        let json = r#"{"data":[{
            "id": 92746535,
            "name": "Luster Pendulum, the Dracoslayer",
            "typeline": ["Dragon", "Pendulum", "Tuner", "Effect"],
            "type": "Pendulum Tuner Effect Monster",
            "humanReadableCardType": "Pendulum Tuner Effect Monster",
            "frameType": "effect_pendulum",
            "desc": "[ Pendulum Effect ] \nOnce per turn, if you have a card in your other Pendulum Zone: You can destroy that card, and if you do, add 1 card from your Deck to your hand, with the same name as that card.\n\n[ Monster Effect ] \nCannot Special Summon Fusion, Synchro, or Xyz Monsters using this card as material, except \"Dracoslayer\" monsters.",
            "race": "Dragon",
            "pend_desc": "Once per turn, if you have a card in your other Pendulum Zone: You can destroy that card, and if you do, add 1 card from your Deck to your hand, with the same name as that card.",
            "monster_desc": "Cannot Special Summon Fusion, Synchro, or Xyz Monsters using this card as material, except \"Dracoslayer\" monsters.",
            "atk": 1850,
            "def": 0,
            "level": 4,
            "attribute": "LIGHT",
            "archetype": "Dracoslayer",
            "scale": 5,
            "ygoprodeck_url": "https://ygoprodeck.com/card/luster-pendulum-the-dracoslayer-7722",
            "card_sets": [
                {
                    "set_name": "Ancient Guardians",
                    "set_code": "ANGU-EN045",
                    "set_rarity": "Rare",
                    "set_rarity_code": "(R)",
                    "set_price": "1.41"
                },
                {
                    "set_name": "Clash of Rebellions",
                    "set_code": "CORE-EN025",
                    "set_rarity": "Super Rare",
                    "set_rarity_code": "(SR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Premium Gold: Infinite Gold",
                    "set_code": "PGL3-EN055",
                    "set_rarity": "Gold Rare",
                    "set_rarity_code": "(GUR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Quarter Century Stampede",
                    "set_code": "RA04-EN252",
                    "set_rarity": "Platinum Secret Rare",
                    "set_rarity_code": "(PS)",
                    "set_price": "0"
                },
                {
                    "set_name": "Quarter Century Stampede",
                    "set_code": "RA04-EN252",
                    "set_rarity": "Quarter Century Secret Rare",
                    "set_rarity_code": "",
                    "set_price": "0"
                }
            ],
            "card_images": [
                {
                    "id": 92746535,
                    "image_url": "https://images.ygoprodeck.com/images/cards/92746535.jpg",
                    "image_url_small": "https://images.ygoprodeck.com/images/cards_small/92746535.jpg",
                    "image_url_cropped": "https://images.ygoprodeck.com/images/cards_cropped/92746535.jpg"
                }
            ],
            "card_prices": [
                {
                    "cardmarket_price": "0.15",
                    "tcgplayer_price": "0.11",
                    "ebay_price": "0.99",
                    "amazon_price": "1.07",
                    "coolstuffinc_price": "0.49"
                }
            ],
            "misc_info": [
                {
                    "views": 261386,
                    "viewsweek": 404,
                    "upvotes": 6,
                    "downvotes": 1,
                    "formats": ["Duel Links", "TCG", "OCG", "Master Duel"],
                    "tcg_date": "2015-08-06",
                    "ocg_date": "2015-04-25",
                    "konami_id": 11809,
                    "has_effect": 1,
                    "md_rarity": "Super Rare"
                }
            ]
        }]}"#;

        let cards = json_to_card_data(json).await.expect("import");
        assert_eq!(cards.len(), 1);

        let card = &cards[0];
        assert_eq!(card.name, "Luster Pendulum, the Dracoslayer");
        assert_eq!(
            card.description,
            "Cannot Special Summon Fusion, Synchro, or Xyz Monsters using this card as material, except \"Dracoslayer\" monsters."
        );
        assert_eq!(card.password.as_deref(), Some("92746535"));
        assert_eq!(card.kind, ygo::CardKind::Monster);
        assert_eq!(card.monster_kind, Some(ygo::MonsterKind::Effect));
        assert_eq!(card.monster_attribute, Some(ygo::MonsterAttribute::Light));
        assert_eq!(card.monster_race, Some(ygo::MonsterRace::Dragon));
        assert_eq!(card.monster_level, Some(4));
        assert_eq!(card.monster_atk, Some(1850));
        assert_eq!(card.monster_def, Some(0));
        assert_eq!(card.monster_pendulum_scale, Some(5));
        assert_eq!(
            card.monster_subtypes,
            Some(vec![ygo::MonsterSubtype::Tuner])
        );
        assert_eq!(
            card.monster_pendulum_effect.as_deref(),
            Some(
                "Once per turn, if you have a card in your other Pendulum Zone: You can destroy that card, and if you do, add 1 card from your Deck to your hand, with the same name as that card."
            )
        );
        assert_eq!(card.monster_link_arrows, None);
    }

    #[tokio::test]
    async fn test_link_monster() {
        let json = r#"{"data":[{
            "id": 60292055,
            "name": "Elphase",
            "typeline": ["Cyberse", "Link", "Effect"],
            "type": "Link Monster",
            "humanReadableCardType": "Link Effect Monster",
            "frameType": "link",
            "desc": "2 Level 3 or higher Cyberse monsters\r\nGains 300 ATK for each monster this card points to. If this Link Summoned card leaves the field: You can target 1 Level 4 or lower Cyberse monster in your GY; Special Summon it, but for the rest of this turn, its effects (if any) are negated and it cannot be used as Link Material. You can only use this effect of \"Elphase\" once per turn.",
            "race": "Cyberse",
            "atk": 2000,
            "def": null,
            "level": null,
            "attribute": "WIND",
            "linkval": 2,
            "linkmarkers": ["Top", "Right"],
            "ygoprodeck_url": "https://ygoprodeck.com/card/elphase-9543",
            "card_sets": [
                {
                    "set_name": "Fists of the Gadgets",
                    "set_code": "FIGA-EN045",
                    "set_rarity": "Super Rare",
                    "set_rarity_code": "(SR)",
                    "set_price": "1.58"
                }
            ],
            "card_images": [
                {
                    "id": 60292055,
                    "image_url": "https://images.ygoprodeck.com/images/cards/60292055.jpg",
                    "image_url_small": "https://images.ygoprodeck.com/images/cards_small/60292055.jpg",
                    "image_url_cropped": "https://images.ygoprodeck.com/images/cards_cropped/60292055.jpg"
                }
            ],
            "card_prices": [
                {
                    "cardmarket_price": "0.09",
                    "tcgplayer_price": "0.17",
                    "ebay_price": "0.99",
                    "amazon_price": "0.25",
                    "coolstuffinc_price": "0.49"
                }
            ],
            "misc_info": [
                {
                    "views": 32981,
                    "viewsweek": 0,
                    "upvotes": 2,
                    "downvotes": 0,
                    "formats": ["TCG", "OCG", "Master Duel"],
                    "tcg_date": "2019-08-22",
                    "ocg_date": "2018-06-23",
                    "konami_id": 13880,
                    "has_effect": 1,
                    "md_rarity": "Rare"
                }
            ]
        }]}"#;

        let cards = json_to_card_data(json).await.expect("import");
        assert_eq!(cards.len(), 1);

        let card = &cards[0];
        assert_eq!(card.name, "Elphase");
        assert_eq!(
            card.description,
            "2 Level 3 or higher Cyberse monsters\r\nGains 300 ATK for each monster this card points to. If this Link Summoned card leaves the field: You can target 1 Level 4 or lower Cyberse monster in your GY; Special Summon it, but for the rest of this turn, its effects (if any) are negated and it cannot be used as Link Material. You can only use this effect of \"Elphase\" once per turn."
        );
        assert_eq!(card.password.as_deref(), Some("60292055"));
        assert_eq!(card.kind, ygo::CardKind::Monster);
        assert_eq!(card.monster_kind, Some(ygo::MonsterKind::Link));
        assert_eq!(card.monster_attribute, Some(ygo::MonsterAttribute::Wind));
        assert_eq!(card.monster_race, Some(ygo::MonsterRace::Cyberse));
        assert_eq!(card.monster_level, Some(2));
        assert_eq!(card.monster_atk, Some(2000));
        assert_eq!(card.monster_def, None);
        assert_eq!(card.monster_pendulum_scale, None);
        assert_eq!(card.monster_pendulum_effect, None);
        assert_eq!(
            card.monster_link_arrows,
            Some(ygo::LinkArrows::Top | ygo::LinkArrows::Right)
        );
    }

    #[tokio::test]
    async fn test_quick_play_spell() {
        let json = r#"{"data":[{
            "id": 93431518,
            "name": "Gateway to Dark World",
            "type": "Spell Card",
            "humanReadableCardType": "Quick-Play Spell",
            "frameType": "spell",
            "desc": "Target 1 \"Dark World\" monster in your Graveyard; Special Summon that target. You cannot Summon other monsters the turn you activate this card (but you can Set).",
            "race": "Quick-Play",
            "archetype": "Dark World",
            "ygoprodeck_url": "https://ygoprodeck.com/card/gateway-to-dark-world-7777",
            "card_sets": [
                {
                    "set_name": "Dark Revelation Volume 4",
                    "set_code": "DR04-EN108",
                    "set_rarity": "Common",
                    "set_rarity_code": "(C)",
                    "set_price": "3.51"
                },
                {
                    "set_name": "Elemental Energy",
                    "set_code": "EEN-EN048",
                    "set_rarity": "Common",
                    "set_rarity_code": "(C)",
                    "set_price": "0"
                },
                {
                    "set_name": "Gates of the Underworld Structure Deck",
                    "set_code": "SDGU-EN025",
                    "set_rarity": "Common",
                    "set_rarity_code": "(C)",
                    "set_price": "1.36"
                },
                {
                    "set_name": "Legendary Collection 4: Joey's World Mega Pack",
                    "set_code": "LCJW-EN250",
                    "set_rarity": "Secret Rare",
                    "set_rarity_code": "(ScR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Structure Deck: Dark World",
                    "set_code": "SR13-EN029",
                    "set_rarity": "Common",
                    "set_rarity_code": "(C)",
                    "set_price": "0"
                }
            ],
            "card_images": [
                {
                    "id": 93431518,
                    "image_url": "https://images.ygoprodeck.com/images/cards/93431518.jpg",
                    "image_url_small": "https://images.ygoprodeck.com/images/cards_small/93431518.jpg",
                    "image_url_cropped": "https://images.ygoprodeck.com/images/cards_cropped/93431518.jpg"
                }
            ],
            "card_prices": [
                {
                    "cardmarket_price": "0.08",
                    "tcgplayer_price": "0.05",
                    "ebay_price": "1.96",
                    "amazon_price": "0.25",
                    "coolstuffinc_price": "0.25"
                }
            ],
            "misc_info": [
                {
                    "views": 104817,
                    "viewsweek": 202,
                    "upvotes": 3,
                    "downvotes": 1,
                    "formats": ["Duel Links", "Common Charity", "Edison", "TCG", "OCG", "Master Duel"],
                    "tcg_date": "2005-11-05",
                    "ocg_date": "2005-08-11",
                    "konami_id": 6515,
                    "has_effect": 1,
                    "md_rarity": "Common"
                }
            ]
        }]}"#;

        let cards = json_to_card_data(json).await.expect("import");
        assert_eq!(cards.len(), 1);

        let card = &cards[0];
        assert_eq!(card.name, "Gateway to Dark World");
        assert_eq!(
            card.description,
            "Target 1 \"Dark World\" monster in your Graveyard; Special Summon that target. You cannot Summon other monsters the turn you activate this card (but you can Set)."
        );
        assert_eq!(card.password.as_deref(), Some("93431518"));
        assert_eq!(card.kind, ygo::CardKind::Spell);
        assert_eq!(card.spell_kind, Some(ygo::SpellKind::QuickPlay));
    }

    #[tokio::test]
    async fn test_counter_trap() {
        let json = r#"{"data":[{
            "id": 41420027,
            "name": "Solemn Judgment",
            "type": "Trap Card",
            "humanReadableCardType": "Counter Trap",
            "frameType": "trap",
            "desc": "When a monster(s) would be Summoned, OR a Spell/Trap Card is activated: Pay half your LP; negate the Summon or activation, and if you do, destroy that card.",
            "race": "Counter",
            "archetype": "Solemn",
            "ygoprodeck_url": "https://ygoprodeck.com/card/solemn-judgment-3537",
            "card_sets": [
                {
                    "set_name": "25th Anniversary Rarity Collection II",
                    "set_code": "RA02-EN075",
                    "set_rarity": "Collector's Rare",
                    "set_rarity_code": "(CR)",
                    "set_price": "0"
                },
                {
                    "set_name": "25th Anniversary Rarity Collection II",
                    "set_code": "RA02-EN075",
                    "set_rarity": "Platinum Secret Rare",
                    "set_rarity_code": "(PS)",
                    "set_price": "0"
                },
                {
                    "set_name": "25th Anniversary Rarity Collection II",
                    "set_code": "RA02-EN075",
                    "set_rarity": "Quarter Century Secret Rare",
                    "set_rarity_code": "",
                    "set_price": "0"
                },
                {
                    "set_name": "25th Anniversary Rarity Collection II",
                    "set_code": "RA02-EN075",
                    "set_rarity": "Secret Rare",
                    "set_rarity_code": "(ScR)",
                    "set_price": "0"
                },
                {
                    "set_name": "25th Anniversary Rarity Collection II",
                    "set_code": "RA02-EN075",
                    "set_rarity": "Super Rare",
                    "set_rarity_code": "(SR)",
                    "set_price": "0"
                },
                {
                    "set_name": "25th Anniversary Rarity Collection II",
                    "set_code": "RA02-EN075",
                    "set_rarity": "Ultimate Rare",
                    "set_rarity_code": "(UtR)",
                    "set_price": "0"
                },
                {
                    "set_name": "25th Anniversary Rarity Collection II",
                    "set_code": "RA02-EN075",
                    "set_rarity": "Ultra Rare",
                    "set_rarity_code": "(UR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Battle Pack: Epic Dawn",
                    "set_code": "BP01-EN047",
                    "set_rarity": "Rare",
                    "set_rarity_code": "(R)",
                    "set_price": "0"
                },
                {
                    "set_name": "Battle Pack: Epic Dawn",
                    "set_code": "BP01-EN047",
                    "set_rarity": "Starfoil Rare",
                    "set_rarity_code": "(SFR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Battles of Legend: Relentless Revenge",
                    "set_code": "BLRR-EN100",
                    "set_rarity": "Ultra Rare",
                    "set_rarity_code": "(UR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Champion Pack: Game One",
                    "set_code": "CP01-EN008",
                    "set_rarity": "Rare",
                    "set_rarity_code": "(R)",
                    "set_price": "0"
                },
                {
                    "set_name": "Dark Beginning 2",
                    "set_code": "DB2-EN073",
                    "set_rarity": "Super Rare",
                    "set_rarity_code": "(SR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Dark Legends",
                    "set_code": "DLG1-EN046",
                    "set_rarity": "Super Rare",
                    "set_rarity_code": "(SR)",
                    "set_price": "21.25"
                },
                {
                    "set_name": "Gold Series 2009",
                    "set_code": "GLD2-EN044",
                    "set_rarity": "Gold Rare",
                    "set_rarity_code": "(GUR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Gold Series: Haunted Mine",
                    "set_code": "GLD5-EN045",
                    "set_rarity": "Ghost/Gold Rare",
                    "set_rarity_code": "(GGR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Legendary Collection 3: Yugi's World Mega Pack",
                    "set_code": "LCYW-EN152",
                    "set_rarity": "Secret Rare",
                    "set_rarity_code": "(ScR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Legendary Collection 4: Joey's World Mega Pack",
                    "set_code": "LCJW-EN182",
                    "set_rarity": "Secret Rare",
                    "set_rarity_code": "(ScR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Lost Sanctuary Structure Deck",
                    "set_code": "SDLS-EN038",
                    "set_rarity": "Common",
                    "set_rarity_code": "(C)",
                    "set_price": "7.43"
                },
                {
                    "set_name": "Maximum Gold",
                    "set_code": "MAGO-EN051",
                    "set_rarity": "Premium Gold Rare",
                    "set_rarity_code": "(PG)",
                    "set_price": "0"
                },
                {
                    "set_name": "Maze of Memories",
                    "set_code": "MAZE-EN063",
                    "set_rarity": "Collector's Rare",
                    "set_rarity_code": "(CR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Maze of Memories",
                    "set_code": "MAZE-EN063",
                    "set_rarity": "Super Rare",
                    "set_rarity_code": "(SR)",
                    "set_price": "0"
                },
                {
                    "set_name": "Metal Raiders",
                    "set_code": "MRD-127",
                    "set_rarity": "Ultra Rare",
                    "set_rarity_code": "(UR)",
                    "set_price": "32.45"
                },
                {
                    "set_name": "Metal Raiders",
                    "set_code": "MRD-E127",
                    "set_rarity": "Ultra Rare",
                    "set_rarity_code": "(UR)",
                    "set_price": "41.99"
                },
                {
                    "set_name": "Metal Raiders",
                    "set_code": "MRD-EN127",
                    "set_rarity": "Ultra Rare",
                    "set_rarity_code": "(UR)",
                    "set_price": "25"
                },
                {
                    "set_name": "OTS Tournament Pack 12",
                    "set_code": "OP12-EN003",
                    "set_rarity": "Ultimate Rare",
                    "set_rarity_code": "(UtR)",
                    "set_price": "0"
                },
                {
                    "set_name": "OTS Tournament Pack 12 (POR)",
                    "set_code": "OP12-PT003",
                    "set_rarity": "Ultimate Rare",
                    "set_rarity_code": "(UtR)",
                    "set_price": "0.00"
                },
                {
                    "set_name": "Retro Pack",
                    "set_code": "RP01-EN045",
                    "set_rarity": "Super Rare",
                    "set_rarity_code": "(SR)",
                    "set_price": "146.79"
                },
                {
                    "set_name": "Secret Slayers",
                    "set_code": "SESL-EN045",
                    "set_rarity": "Super Rare",
                    "set_rarity_code": "(SR)",
                    "set_price": "6.6"
                },
                {
                    "set_name": "Structure Deck: Fire Kings",
                    "set_code": "SR14-EN038",
                    "set_rarity": "Common",
                    "set_rarity_code": "(C)",
                    "set_price": "0"
                },
                {
                    "set_name": "The Lost Art Promotion (series)",
                    "set_code": "LART-EN014",
                    "set_rarity": "Common",
                    "set_rarity_code": "(C)",
                    "set_price": "0"
                },
                {
                    "set_name": "The Lost Art Promotion N",
                    "set_code": "LART-EN014",
                    "set_rarity": "Ultra Rare",
                    "set_rarity_code": "(UR)",
                    "set_price": "0"
                }
            ],
            "card_images": [
                {
                    "id": 41420027,
                    "image_url": "https://images.ygoprodeck.com/images/cards/41420027.jpg",
                    "image_url_small": "https://images.ygoprodeck.com/images/cards_small/41420027.jpg",
                    "image_url_cropped": "https://images.ygoprodeck.com/images/cards_cropped/41420027.jpg"
                }
            ],
            "card_prices": [
                {
                    "cardmarket_price": "0.57",
                    "tcgplayer_price": "0.82",
                    "ebay_price": "3.99",
                    "amazon_price": "7.39",
                    "coolstuffinc_price": "3.99"
                }
            ],
            "misc_info": [
                {
                    "beta_name": "Solemn Judgement",
                    "staple": "Yes",
                    "views": 3441019,
                    "viewsweek": 11012,
                    "upvotes": 146,
                    "downvotes": 22,
                    "formats": ["GOAT", "OCG GOAT", "Common Charity", "Edison", "TCG", "OCG", "Master Duel"],
                    "tcg_date": "2002-06-26",
                    "ocg_date": "1999-11-18",
                    "konami_id": 4861,
                    "has_effect": 1,
                    "md_rarity": "Ultra Rare"
                }
            ]
        }]}"#;

        let cards = json_to_card_data(json).await.expect("import");
        assert_eq!(cards.len(), 1);

        let card = &cards[0];
        assert_eq!(card.name, "Solemn Judgment");
        assert_eq!(
            card.description,
            "When a monster(s) would be Summoned, OR a Spell/Trap Card is activated: Pay half your LP; negate the Summon or activation, and if you do, destroy that card."
        );
        assert_eq!(card.password.as_deref(), Some("41420027"));
        assert_eq!(card.kind, ygo::CardKind::Trap);
        assert_eq!(card.trap_kind, Some(ygo::TrapKind::Counter));
    }

    #[tokio::test]
    async fn test_import_json() {
        with_db_pool(async move |db_pool| {
            let json = r#"{"data":[{
                "id": 46533533,
                "name": "Dipity",
                "typeline": ["Fiend", "Normal"],
                "type": "Normal Monster",
                "humanReadableCardType": "Normal Monster",
                "frameType": "normal",
                "desc": "''A cute little thing who lives in a glass bottle and is said to bring good luck to the one who makes a contract with it. The lid must never be opened, no matter what happens...''",
                "race": "Fiend",
                "atk": 0,
                "def": 0,
                "level": 4,
                "attribute": "LIGHT",
                "ygoprodeck_url": "https://ygoprodeck.com/card/dipity-14840",
                "card_sets": [
                    {
                        "set_name": "Alliance Insight",
                        "set_code": "ALIN-EN097",
                        "set_rarity": "Common",
                        "set_rarity_code": "(C)",
                        "set_price": "0"
                    }
                ],
                "card_images": [
                    {
                        "id": 46533533,
                        "image_url": "https://images.ygoprodeck.com/images/cards/46533533.jpg",
                        "image_url_small": "https://images.ygoprodeck.com/images/cards_small/46533533.jpg",
                        "image_url_cropped": "https://images.ygoprodeck.com/images/cards_cropped/46533533.jpg"
                    }
                ],
                "card_prices": [
                    {
                        "cardmarket_price": "0.11",
                        "tcgplayer_price": "0.11",
                        "ebay_price": "0.00",
                        "amazon_price": "0.00",
                        "coolstuffinc_price": "0.00"
                    }
                ],
                "misc_info": [
                    {
                        "beta_name": "Pity",
                        "views": 13534,
                        "viewsweek": 505,
                        "upvotes": 0,
                        "downvotes": 0,
                        "formats": ["Common Charity", "TCG", "OCG", "Master Duel"],
                        "tcg_date": "2025-05-01",
                        "ocg_date": "2024-04-01",
                        "konami_id": 20274,
                        "has_effect": 1,
                        "md_rarity": "Super Rare"
                    }
                ]
            }]}"#;

            let client = db_pool.get().await.expect("Could not get DB client");
            let (inserted, updated) = import_from_json_str(&client, json)
                .await
                .expect("Could not import cards from JSON");

            assert_eq!(inserted, 1);
            assert_eq!(updated, 0);

            let card = service::card::get_by_konami_id(&client, 20274)
                .await
                .expect("Could not get card by Konami ID")
                .expect("Card not found");

            assert_eq!(card.data.name, "Dipity");
            assert_eq!(card.data.kind, ygo::CardKind::Monster);
            assert_eq!(card.data.monster_race, Some(ygo::MonsterRace::Fiend));
            assert_eq!(
                card.data.monster_attribute,
                Some(ygo::MonsterAttribute::Light)
            );
            assert_eq!(card.data.monster_level, Some(4));
            assert_eq!(card.data.monster_atk, Some(0));
            assert_eq!(card.data.monster_def, Some(0));
        }).await
    }

    #[tokio::test]
    async fn test_import_json_overrides_by_konami_id() {
        with_db_pool(async move |db_pool| {
            let json = r#"{"data":[{
                "id": 46533533,
                "name": "Dipity",
                "typeline": ["Fiend", "Normal"],
                "type": "Normal Monster",
                "humanReadableCardType": "Normal Monster",
                "frameType": "normal",
                "desc": "''A cute little thing who lives in a glass bottle and is said to bring good luck to the one who makes a contract with it. The lid must never be opened, no matter what happens...''",
                "race": "Fiend",
                "atk": 0,
                "def": 0,
                "level": 4,
                "attribute": "LIGHT",
                "ygoprodeck_url": "https://ygoprodeck.com/card/dipity-14840",
                "card_sets": [
                    {
                        "set_name": "Alliance Insight",
                        "set_code": "ALIN-EN097",
                        "set_rarity": "Common",
                        "set_rarity_code": "(C)",
                        "set_price": "0"
                    }
                ],
                "card_images": [
                    {
                        "id": 46533533,
                        "image_url": "https://images.ygoprodeck.com/images/cards/46533533.jpg",
                        "image_url_small": "https://images.ygoprodeck.com/images/cards_small/46533533.jpg",
                        "image_url_cropped": "https://images.ygoprodeck.com/images/cards_cropped/46533533.jpg"
                    }
                ],
                "card_prices": [
                    {
                        "cardmarket_price": "0.11",
                        "tcgplayer_price": "0.11",
                        "ebay_price": "0.00",
                        "amazon_price": "0.00",
                        "coolstuffinc_price": "0.00"
                    }
                ],
                "misc_info": [
                    {
                        "beta_name": "Pity",
                        "views": 13534,
                        "viewsweek": 505,
                        "upvotes": 0,
                        "downvotes": 0,
                        "formats": ["Common Charity", "TCG", "OCG", "Master Duel"],
                        "tcg_date": "2025-05-01",
                        "ocg_date": "2024-04-01",
                        "konami_id": 20274,
                        "has_effect": 1,
                        "md_rarity": "Super Rare"
                    }
                ]
            }]}"#;

            let client = db_pool.get().await.expect("Could not get DB client");

            let existing_card = ygo::NewCard {
                data: CardData {
                    name: "Dipity Old".to_string(),
                    konami_id: Some(20274),
                    ..Default::default()
                }
            };
            service::card::save_new(&client, &existing_card)
                .await
                .expect("Could not save existing card");

            let (inserted, updated) = import_from_json_str(&client, json)
                .await
                .expect("Could not import cards from JSON");

            assert_eq!(inserted, 0);
            assert_eq!(updated, 1);

            let card = service::card::get_by_konami_id(&client, 20274)
                .await
                .expect("Could not get card by Konami ID")
                .expect("Card not found");

            assert_eq!(card.data.name, "Dipity");
            assert_eq!(card.data.kind, ygo::CardKind::Monster);
            assert_eq!(card.data.monster_race, Some(ygo::MonsterRace::Fiend));
            assert_eq!(
                card.data.monster_attribute,
                Some(ygo::MonsterAttribute::Light)
            );
            assert_eq!(card.data.monster_level, Some(4));
            assert_eq!(card.data.monster_atk, Some(0));
            assert_eq!(card.data.monster_def, Some(0));
        }).await
    }

    #[tokio::test]
    async fn test_import_json_overrides_by_password() {
        with_db_pool(async move |db_pool| {
            let json = r#"{"data":[{
                "id": 46533533,
                "name": "Dipity",
                "typeline": ["Fiend", "Normal"],
                "type": "Normal Monster",
                "humanReadableCardType": "Normal Monster",
                "frameType": "normal",
                "desc": "''A cute little thing who lives in a glass bottle and is said to bring good luck to the one who makes a contract with it. The lid must never be opened, no matter what happens...''",
                "race": "Fiend",
                "atk": 0,
                "def": 0,
                "level": 4,
                "attribute": "LIGHT",
                "ygoprodeck_url": "https://ygoprodeck.com/card/dipity-14840",
                "card_sets": [
                    {
                        "set_name": "Alliance Insight",
                        "set_code": "ALIN-EN097",
                        "set_rarity": "Common",
                        "set_rarity_code": "(C)",
                        "set_price": "0"
                    }
                ],
                "card_images": [
                    {
                        "id": 46533533,
                        "image_url": "https://images.ygoprodeck.com/images/cards/46533533.jpg",
                        "image_url_small": "https://images.ygoprodeck.com/images/cards_small/46533533.jpg",
                        "image_url_cropped": "https://images.ygoprodeck.com/images/cards_cropped/46533533.jpg"
                    }
                ],
                "card_prices": [
                    {
                        "cardmarket_price": "0.11",
                        "tcgplayer_price": "0.11",
                        "ebay_price": "0.00",
                        "amazon_price": "0.00",
                        "coolstuffinc_price": "0.00"
                    }
                ],
                "misc_info": [
                    {
                        "beta_name": "Pity",
                        "views": 13534,
                        "viewsweek": 505,
                        "upvotes": 0,
                        "downvotes": 0,
                        "formats": ["Common Charity", "TCG", "OCG", "Master Duel"],
                        "tcg_date": "2025-05-01",
                        "ocg_date": "2024-04-01",
                        "has_effect": 1,
                        "md_rarity": "Super Rare"
                    }
                ]
            }]}"#;

            let client = db_pool.get().await.expect("Could not get DB client");

            let existing_card = ygo::NewCard {
                data: CardData {
                    name: "Dipity Old".to_string(),
                    password: Some("46533533".to_string()),
                    ..Default::default()
                }
            };
            service::card::save_new(&client, &existing_card)
                .await
                .expect("Could not save existing card");

            let (inserted, updated) = import_from_json_str(&client, json)
                .await
                .expect("Could not import cards from JSON");

            assert_eq!(inserted, 0);
            assert_eq!(updated, 1);

            let card = service::card::get_by_password(&client, "46533533")
                .await
                .expect("Could not get card by password")
                .expect("Card not found");

            assert_eq!(card.data.name, "Dipity");
            assert_eq!(card.data.kind, ygo::CardKind::Monster);
            assert_eq!(card.data.monster_race, Some(ygo::MonsterRace::Fiend));
            assert_eq!(
                card.data.monster_attribute,
                Some(ygo::MonsterAttribute::Light)
            );
            assert_eq!(card.data.monster_level, Some(4));
            assert_eq!(card.data.monster_atk, Some(0));
            assert_eq!(card.data.monster_def, Some(0));
        }).await
    }

    #[tokio::test]
    async fn test_import_does_not_insert_if_konami_id_and_password_are_missing() {
        with_db_pool(async move |db_pool| {
            let json = r#"{"data":[{
                "id": 146533533,
                "name": "Dipity",
                "typeline": ["Fiend", "Normal"],
                "type": "Normal Monster",
                "humanReadableCardType": "Normal Monster",
                "frameType": "normal",
                "desc": "''A cute little thing who lives in a glass bottle and is said to bring good luck to the one who makes a contract with it. The lid must never be opened, no matter what happens...''",
                "race": "Fiend",
                "atk": 0,
                "def": 0,
                "level": 4,
                "attribute": "LIGHT",
                "ygoprodeck_url": "https://ygoprodeck.com/card/dipity-14840",
                "card_sets": [
                    {
                        "set_name": "Alliance Insight",
                        "set_code": "ALIN-EN097",
                        "set_rarity": "Common",
                        "set_rarity_code": "(C)",
                        "set_price": "0"
                    }
                ],
                "card_images": [
                    {
                        "id": 46533533,
                        "image_url": "https://images.ygoprodeck.com/images/cards/46533533.jpg",
                        "image_url_small": "https://images.ygoprodeck.com/images/cards_small/46533533.jpg",
                        "image_url_cropped": "https://images.ygoprodeck.com/images/cards_cropped/46533533.jpg"
                    }
                ],
                "card_prices": [
                    {
                        "cardmarket_price": "0.11",
                        "tcgplayer_price": "0.11",
                        "ebay_price": "0.00",
                        "amazon_price": "0.00",
                        "coolstuffinc_price": "0.00"
                    }
                ],
                "misc_info": [
                    {
                        "beta_name": "Pity",
                        "views": 13534,
                        "viewsweek": 505,
                        "upvotes": 0,
                        "downvotes": 0,
                        "formats": ["Common Charity", "TCG", "OCG", "Master Duel"],
                        "tcg_date": "2025-05-01",
                        "ocg_date": "2024-04-01",
                        "has_effect": 1,
                        "md_rarity": "Super Rare"
                    }
                ]
            }]}"#;

            let client = db_pool.get().await.expect("Could not get DB client");

            let (inserted, updated) = import_from_json_str(&client, json)
                .await
                .expect("Could not import cards from JSON");

            assert_eq!(inserted, 0);
            assert_eq!(updated, 0);

            let cards = service::card::get_all(&client)
                .await
                .expect("Could not get cards");

            assert_eq!(cards.len(), 0);
        }).await
    }
}
