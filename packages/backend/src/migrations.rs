use crate::database::Migration;

pub const MIGRATIONS: &[Migration] = &[(
    "250803_01__yugioh_card",
    include_str!("migrations/250803_01_up__yugioh_card.sql"),
    Some(include_str!("migrations/250803_01_dn__yugioh_card.sql")),
)];
