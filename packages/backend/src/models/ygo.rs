use std::fmt;

use bitflags::bitflags;
use chrono::{DateTime, NaiveDate, Utc};
use postgres_types::{FromSql, ToSql};
use rust_decimal::Decimal;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, SeqAccess, Visitor},
    ser::SerializeSeq,
};

/// A Yu-Gi-Oh! card in the database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Card {
    pub id: i32,
    pub updated_at: DateTime<Utc>,
    #[serde(flatten)]
    pub data: CardData,
}

/// Represents card informations.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CardData {
    pub name: String,
    pub description: String,
    pub kind: CardKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub konami_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub treated_as: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcg_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocg_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcgplayer_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cardmarket_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ebay_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coolstuffinc_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster_kind: Option<MonsterKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster_attribute: Option<MonsterAttribute>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster_race: Option<MonsterRace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster_subtypes: Option<Vec<MonsterSubtype>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster_atk: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster_def: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster_level: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster_pendulum_scale: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster_pendulum_effect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monster_link_arrows: Option<LinkArrows>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spell_kind: Option<SpellKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trap_kind: Option<TrapKind>,
}

/// Card kinds (Monster, Spell, Trap)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, ToSql, FromSql)]
#[postgres(name = "ygo_card_kind", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CardKind {
    #[default]
    Monster,
    Spell,
    Trap,
}

/// Monster card types (Token, Normal, Effect, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "ygo_monster_kind", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MonsterKind {
    #[default]
    Other,
    Token,
    Normal,
    Effect,
    Fusion,
    Ritual,
    Synchro,
    Xyz,
    Link,
}

/// Monster card subtypes (Flip, Gemini, Spirit, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "ygo_monster_subtype", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MonsterSubtype {
    #[default]
    Other,
    Flip,
    Gemini,
    Spirit,
    Toon,
    Tuner,
    Union,
}

/// Monster card attributes (Dark, Divine, Earth, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "ygo_monster_attribute", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MonsterAttribute {
    #[default]
    Other,
    Dark,
    Divine,
    Earth,
    Fire,
    Light,
    Water,
    Wind,
}

/// Monster types (Aqua, Beast, BeastWarrior, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "ygo_monster_race", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MonsterRace {
    #[default]
    Other,
    Aqua,
    Beast,
    BeastWarrior,
    CreatorGod,
    Cyberse,
    Dinosaur,
    DivineBeast,
    Dragon,
    Fairy,
    Fiend,
    Fish,
    Illusion,
    Insect,
    Machine,
    Plant,
    Psychic,
    Pyro,
    Reptile,
    Rock,
    SeaSerpent,
    Spellcaster,
    Thunder,
    Warrior,
    WingedBeast,
    Wyrm,
    Zombie,
}

#[derive(Debug, Clone, PartialEq, Copy, ToSql, FromSql)]
pub struct LinkArrows(pub i16);

bitflags! {
    impl LinkArrows: i16 {
        const TopLeft = 1 << 0;
        const Top = 1 << 1;
        const TopRight = 1 << 2;
        const Left = 1 << 3;
        const Right = 1 << 4;
        const BottomLeft = 1 << 5;
        const Bottom = 1 << 6;
        const BottomRight = 1 << 7;
    }
}

const LINK_ARROW_LABELS: [(LinkArrows, &str); 8] = [
    (LinkArrows::TopLeft, "top_left"),
    (LinkArrows::Top, "top"),
    (LinkArrows::TopRight, "top_right"),
    (LinkArrows::Left, "left"),
    (LinkArrows::Right, "right"),
    (LinkArrows::BottomLeft, "bottom_left"),
    (LinkArrows::Bottom, "bottom"),
    (LinkArrows::BottomRight, "bottom_right"),
];

impl Serialize for LinkArrows {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let flags: Vec<&str> = LINK_ARROW_LABELS
            .iter()
            .filter_map(|&(flag, name)| self.contains(flag).then_some(name))
            .collect();

        let mut seq = serializer.serialize_seq(Some(flags.len()))?;
        for flag in flags {
            seq.serialize_element(flag)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for LinkArrows {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FlagsVisitor;

        impl<'de> Visitor<'de> for FlagsVisitor {
            type Value = LinkArrows;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an array of link arrows (top_left, top, top_right, left, right, bottom_left, bottom, bottom_right)")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut flags = LinkArrows::empty();
                while let Some(flag) = seq.next_element::<String>()? {
                    match LINK_ARROW_LABELS.iter().find(|&&(_, name)| name == flag) {
                        Some(&(flag, _)) => flags |= flag,
                        None => {
                            return Err(de::Error::unknown_variant(
                                &flag,
                                &[
                                    "top_left",
                                    "top",
                                    "top_right",
                                    "left",
                                    "right",
                                    "bottom_left",
                                    "bottom",
                                    "bottom_right",
                                ],
                            ));
                        }
                    }
                }
                Ok(flags)
            }
        }

        deserializer.deserialize_seq(FlagsVisitor)
    }
}

/// Spell card kinds (Normal, Continuous, Field, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "ygo_spell_kind", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SpellKind {
    #[default]
    Other,
    Normal,
    Continuous,
    Field,
    Equip,
    Ritual,
    QuickPlay,
}

/// Trap card kinds (Normal, Continous, Counter)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, ToSql, FromSql)]
#[postgres(name = "ygo_trap_kind", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TrapKind {
    #[default]
    Other,
    Normal,
    Continuous,
    Counter,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_markers_serialize_to_json_array() {
        let link_markers = LinkArrows::TopLeft | LinkArrows::BottomRight;
        let json = serde_json::to_string(&link_markers).unwrap();

        assert_eq!(json, r#"["top_left","bottom_right"]"#);
    }

    #[test]
    fn test_link_markers_deserialize_from_json_array() {
        let json_array = r#"["top_right", "left", "bottom_left", "bottom"]"#;
        let link_markers = serde_json::from_str::<LinkArrows>(json_array).unwrap();

        assert_eq!(
            link_markers,
            LinkArrows::TopRight | LinkArrows::Left | LinkArrows::BottomLeft | LinkArrows::Bottom
        );
    }

    #[test]
    fn test_fails_when_deserializing_invalid_value() {
        let json = r#""invalid""#;
        let result = serde_json::from_str::<LinkArrows>(json);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid type: string \"invalid\", expected an array of link arrows (top_left, top, top_right, left, right, bottom_left, bottom, bottom_right) at line 1 column 9"
        );
    }

    #[test]
    fn test_fails_when_deserializing_invalid_arrow_values() {
        let json_array = r#"["top_right", "left", "bottom_left", "bottom", "invalid"]"#;
        let result = serde_json::from_str::<LinkArrows>(json_array);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "unknown variant `invalid`, expected one of `top_left`, `top`, `top_right`, `left`, `right`, `bottom_left`, `bottom`, `bottom_right` at line 1 column 57"
        );
    }
}
