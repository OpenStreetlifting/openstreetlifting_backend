ALTER TABLE competition_participants ADD COLUMN category_id UUID;
ALTER TABLE competition_participants ADD COLUMN competition_id UUID;

UPDATE competition_participants cp
SET
    category_id = cg.category_id,
    competition_id = cg.competition_id
FROM competition_groups cg
WHERE cp.group_id = cg.group_id;

ALTER TABLE competition_participants ALTER COLUMN category_id SET NOT NULL;
ALTER TABLE competition_participants ALTER COLUMN competition_id SET NOT NULL;

ALTER TABLE competition_participants DROP CONSTRAINT competition_participants_group_id_fkey;

DROP INDEX IF EXISTS competition_participants_index_0;
DROP INDEX IF EXISTS competition_participants_index_3;

ALTER TABLE competition_participants
ADD CONSTRAINT competition_participants_category_id_fkey
FOREIGN KEY(category_id) REFERENCES categories(category_id) ON UPDATE CASCADE ON DELETE RESTRICT;

ALTER TABLE competition_participants
ADD CONSTRAINT competition_participants_competition_id_fkey
FOREIGN KEY(competition_id) REFERENCES competitions(competition_id) ON UPDATE CASCADE ON DELETE CASCADE;

CREATE INDEX competition_participants_category_rank_idx ON competition_participants (category_id, rank);
CREATE INDEX competition_participants_competition_idx ON competition_participants (competition_id);
CREATE UNIQUE INDEX competition_participants_unique_idx ON competition_participants (competition_id, category_id, athlete_id);

ALTER TABLE competition_participants DROP COLUMN group_id;

DROP TABLE competition_groups CASCADE;
