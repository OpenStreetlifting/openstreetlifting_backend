-- RIS Computation Tables Migration
-- This migration adds support for computing RIS (Relative Index for Streetlifting) scores
-- internally using versioned formulas, rather than relying on imported scores.

-- Table: ris_formula_versions
-- Stores the formula constants for each year/version of the RIS calculation
CREATE TABLE IF NOT EXISTS "ris_formula_versions" (
    "formula_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    "year" INTEGER NOT NULL UNIQUE,
    "effective_from" DATE NOT NULL,
    "effective_until" DATE,
    "is_current" BOOLEAN NOT NULL DEFAULT FALSE,

    -- Men's constants (A, K, B, v, Q)
    "men_a" DECIMAL(10,5) NOT NULL,
    "men_k" DECIMAL(10,5) NOT NULL,
    "men_b" DECIMAL(10,5) NOT NULL,
    "men_v" DECIMAL(10,5) NOT NULL,
    "men_q" DECIMAL(10,5) NOT NULL,

    -- Women's constants (A, K, B, v, Q)
    "women_a" DECIMAL(10,5) NOT NULL,
    "women_k" DECIMAL(10,5) NOT NULL,
    "women_b" DECIMAL(10,5) NOT NULL,
    "women_v" DECIMAL(10,5) NOT NULL,
    "women_q" DECIMAL(10,5) NOT NULL,

    "notes" TEXT,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY("formula_id"),
    CONSTRAINT "valid_effective_period" CHECK (effective_until IS NULL OR effective_until > effective_from)
);

CREATE INDEX "ris_formula_versions_index_0" ON "ris_formula_versions" ("year");
CREATE INDEX "ris_formula_versions_index_1" ON "ris_formula_versions" ("is_current");
CREATE INDEX "ris_formula_versions_index_2" ON "ris_formula_versions" ("effective_from", "effective_until");

-- Table: ris_scores_history
-- Stores computed RIS scores with full version history for each participant
CREATE TABLE IF NOT EXISTS "ris_scores_history" (
    "ris_score_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    "participant_id" UUID NOT NULL,
    "formula_id" UUID NOT NULL,
    "ris_score" DECIMAL(10,2) NOT NULL,
    "bodyweight" DECIMAL(6,2) NOT NULL,
    "total_weight" DECIMAL(8,2) NOT NULL,
    "computed_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY("ris_score_id")
);

CREATE INDEX "ris_scores_history_index_0" ON "ris_scores_history" ("participant_id", "formula_id");
CREATE INDEX "ris_scores_history_index_1" ON "ris_scores_history" ("participant_id");
CREATE INDEX "ris_scores_history_index_2" ON "ris_scores_history" ("formula_id");
CREATE UNIQUE INDEX "ris_scores_history_index_3" ON "ris_scores_history" ("participant_id", "formula_id");

-- Foreign Keys
ALTER TABLE "ris_scores_history"
ADD FOREIGN KEY("participant_id") REFERENCES "competition_participants"("participant_id") ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE "ris_scores_history"
ADD FOREIGN KEY("formula_id") REFERENCES "ris_formula_versions"("formula_id") ON UPDATE CASCADE ON DELETE RESTRICT;

-- Seed the 2025 RIS formula
-- Formula: RIS = Total × 100 / (A + (K - A) / (1 + Q · e^(-B · (BW - v))))
-- Source: RIS 2025 Edition - Created by Waris Radji & Mathieu Ardoin
INSERT INTO ris_formula_versions (
    year, effective_from, is_current,
    men_a, men_k, men_b, men_v, men_q,
    women_a, women_k, women_b, women_v, women_q,
    notes
) VALUES (
    2025,
    '2025-01-01',
    TRUE,
    338.00000, 549.00000, 0.11354, 74.77700, 0.53096,
    164.00000, 270.00000, 0.13776, 57.85500, 0.37089,
    'RIS 2025 Edition - Created by Waris Radji & Mathieu Ardoin'
);
