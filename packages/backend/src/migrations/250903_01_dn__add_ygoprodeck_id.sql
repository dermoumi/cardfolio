DO $$ BEGIN
    ALTER TABLE ygo_cards DROP COLUMN ygoprodeck_id;
END $$;
