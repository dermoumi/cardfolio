use crate::database::Migration;

pub const MIGRATIONS: &[Migration] = &[
    (
        "250803_01__yugioh_card",
        include_str!("migrations/250803_01_up__yugioh_card.sql"),
        Some(include_str!("migrations/250803_01_dn__yugioh_card.sql")),
    ),
    (
        "250903_01__add_ygoprodeck_id",
        include_str!("migrations/250903_01_up__add_ygoprodeck_id.sql"),
        Some(include_str!(
            "migrations/250903_01_dn__add_ygoprodeck_id.sql"
        )),
    ),
];
