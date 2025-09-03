DO $$ BEGIN
    ALTER TABLE ygo_cards ADD COLUMN ygoprodeck_id INTEGER;
END $$;
