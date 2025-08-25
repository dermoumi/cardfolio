DO $$ BEGIN
    CREATE TYPE YGO_CARD_KIND AS ENUM('monster', 'spell', 'trap');

    CREATE TYPE YGO_MONSTER_KIND AS ENUM(
        'other',
        'token',
        'normal',
        'effect',
        'fusion',
        'ritual',
        'synchro',
        'xyz',
        'link'
    );

    CREATE TYPE YGO_MONSTER_SUBTYPE AS ENUM(
        'other',
        'flip',
        'gemini',
        'spirit',
        'toon',
        'tuner',
        'union'
    );

    CREATE TYPE YGO_MONSTER_ATTRIBUTE AS ENUM(
        'other',
        'dark',
        'divine',
        'earth',
        'fire',
        'light',
        'water',
        'wind'
    );

    CREATE TYPE YGO_MONSTER_RACE AS ENUM(
        'other',
        'aqua',
        'beast',
        'beast_warrior',
        'creator_god',
        'cyberse',
        'dinosaur',
        'divine_beast',
        'dragon',
        'fairy',
        'fiend',
        'fish',
        'illusion',
        'insect',
        'machine',
        'plant',
        'psychic',
        'pyro',
        'reptile',
        'rock',
        'sea_serpent',
        'spellcaster',
        'thunder',
        'warrior',
        'winged_beast',
        'wyrm',
        'zombie'
    );

    CREATE TYPE YGO_SPELL_KIND AS ENUM(
        'other',
        'normal',
        'continuous',
        'field',
        'equip',
        'ritual',
        'quick_play'
    );

    CREATE TYPE YGO_TRAP_KIND AS ENUM(
        'other',
        'normal',
        'continuous',
        'counter'
    );

    CREATE TABLE IF NOT EXISTS
        ygo_cards (
            id SERIAL PRIMARY KEY,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,

            -- Common fields
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            kind YGO_CARD_KIND NOT NULL,
            password TEXT,
            konami_id INTEGER,
            treated_as INTEGER REFERENCES ygo_cards (id) ON DELETE SET NULL,
            tcg_date DATE,
            ocg_date DATE,

            -- Monster-related fields
            monster_kind YGO_MONSTER_KIND,
            monster_attribute YGO_MONSTER_ATTRIBUTE,
            monster_race YGO_MONSTER_RACE,
            monster_subtypes YGO_MONSTER_SUBTYPE[],
            monster_atk SMALLINT,
            monster_def SMALLINT,
            monster_level SMALLINT,
            monster_pendulum_scale SMALLINT,
            monster_pendulum_effect TEXT,
            monster_link_arrows SMALLINT,

            -- Spell-related fields
            spell_kind YGO_SPELL_KIND,

            -- Trap-related fields
            trap_kind YGO_TRAP_KIND,

            CONSTRAINT ygo_cards_unique_konami_id UNIQUE (konami_id)
        );

    CREATE INDEX IF NOT EXISTS ygo_cards_name_idx ON ygo_cards (name);
    CREATE INDEX IF NOT EXISTS ygo_cards_monster_atk_idx ON ygo_cards (monster_atk);
    CREATE INDEX IF NOT EXISTS ygo_cards_monster_def_idx ON ygo_cards (monster_def);
    CREATE INDEX IF NOT EXISTS ygo_cards_monster_level_idx ON ygo_cards (monster_level);
    CREATE INDEX IF NOT EXISTS ygo_cards_password_idx ON ygo_cards (password);
    CREATE INDEX IF NOT EXISTS ygo_cards_tcg_date_idx ON ygo_cards (tcg_date);
    CREATE INDEX IF NOT EXISTS ygo_cards_ocg_date_idx ON ygo_cards (ocg_date);
END $$;
