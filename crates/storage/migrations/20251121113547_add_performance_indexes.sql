-- Migration: Add performance indexes for common query patterns
-- Date: 2025-11-21
-- Purpose: Optimize database queries by adding missing indexes

-- Index on competitions.federation_id for JOIN operations
-- This helps when fetching competition details with federation info
CREATE INDEX IF NOT EXISTS "competitions_index_federation"
ON "competitions" ("federation_id");

-- Index on lifts for better sorting by movement and participant
-- Helps when ordering lifts for display
CREATE INDEX IF NOT EXISTS "lifts_index_participant_movement"
ON "lifts" ("participant_id", "movement_name", "max_weight");

-- Index on attempts for better lift_id + attempt_number lookups
-- The existing unique index covers this, but explicit index for clarity
-- (Commenting out as unique index on (lift_id, attempt_number) already exists)
-- CREATE INDEX IF NOT EXISTS "attempts_index_lift_attempt"
-- ON "attempts" ("lift_id", "attempt_number");

-- Index for filtering participants by disqualification status
CREATE INDEX IF NOT EXISTS "competition_participants_index_disqualified"
ON "competition_participants" ("is_disqualified")
WHERE "is_disqualified" = true;

-- Partial index for active (non-cancelled) competitions
-- Improves performance when listing active competitions
CREATE INDEX IF NOT EXISTS "competitions_index_active"
ON "competitions" ("start_date", "status")
WHERE "status" IN ('draft', 'upcoming', 'live', 'completed');

-- Index on federation country for filtering
CREATE INDEX IF NOT EXISTS "federations_index_country"
ON "federations" ("country")
WHERE "country" IS NOT NULL;

-- Composite index for competition groups lookup
-- Helps when fetching all groups for a competition with category info
CREATE INDEX IF NOT EXISTS "competition_groups_index_comp_category"
ON "competition_groups" ("competition_id", "category_id");
