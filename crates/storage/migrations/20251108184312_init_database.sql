-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS "athletes" (
	"athlete_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
	"first_name" VARCHAR(255) NOT NULL,
	"last_name" VARCHAR(255) NOT NULL,
	"gender" VARCHAR(10) NOT NULL CHECK (gender IN ('M', 'F', 'MX')),
	"created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	"nationality" VARCHAR(255),
	"country" VARCHAR(255) NOT NULL,
	"profile_picture_url" VARCHAR(500),
	PRIMARY KEY("athlete_id")
);

CREATE INDEX "athletes_index_0" ON "athletes" ("last_name", "first_name");
CREATE INDEX "athletes_index_1" ON "athletes" ("gender");

CREATE TABLE IF NOT EXISTS "competitions" (
	"competition_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
	"name" VARCHAR(255) NOT NULL,
	"created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	"slug" VARCHAR(255) NOT NULL UNIQUE,
	"status" VARCHAR(50) NOT NULL CHECK (status IN ('draft', 'upcoming', 'live', 'completed', 'cancelled')),
	"federation_id" UUID NOT NULL,
	"venue" VARCHAR(255),
	"city" VARCHAR(255),
	"country" VARCHAR(255),
	"start_date" DATE NOT NULL,
	"end_date" DATE NOT NULL,
	"number_of_judge" SMALLINT CHECK (number_of_judge IN (1, 3)),
	PRIMARY KEY("competition_id"),
	CONSTRAINT "valid_date_range" CHECK (end_date >= start_date)
);

CREATE INDEX "competitions_index_0" ON "competitions" ("start_date");
CREATE INDEX "competitions_index_1" ON "competitions" ("status");
CREATE INDEX "competitions_index_2" ON "competitions" ("slug");
CREATE INDEX "competitions_index_3" ON "competitions" ("country", "city");

CREATE TABLE IF NOT EXISTS "categories" (
	"category_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
	"name" VARCHAR(255) NOT NULL,
	"gender" VARCHAR(10) NOT NULL CHECK (gender IN ('M', 'F', 'MX')),
	"weight_class_min" DECIMAL CHECK (weight_class_min > 0),
	"weight_class_max" DECIMAL CHECK (weight_class_max > 0),
	PRIMARY KEY("category_id"),
	CONSTRAINT "valid_weight_range" CHECK (weight_class_max > weight_class_min OR (weight_class_min IS NULL AND weight_class_max IS NOT NULL))
);

CREATE INDEX "categories_index_0" ON "categories" ("gender");
CREATE INDEX "categories_index_1" ON "categories" ("weight_class_min", "weight_class_max");

CREATE TABLE IF NOT EXISTS "competition_participants" (
	"participant_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
	"group_id" UUID NOT NULL,
	"athlete_id" UUID NOT NULL,
	"bodyweight" DECIMAL CHECK (bodyweight > 0),
	"rank" INTEGER CHECK (rank > 0),
	"is_disqualified" BOOLEAN NOT NULL DEFAULT FALSE,
	"created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	"disqualified_reason" TEXT,
	"ris_score" DECIMAL,
	PRIMARY KEY("participant_id")
);

CREATE INDEX "competition_participants_index_0" ON "competition_participants" ("group_id", "rank");
CREATE INDEX "competition_participants_index_1" ON "competition_participants" ("athlete_id");
CREATE UNIQUE INDEX "competition_participants_index_3" ON "competition_participants" ("group_id", "athlete_id");

CREATE TABLE IF NOT EXISTS "movements" (
	"name" VARCHAR(255) NOT NULL,
	"display_order" INTEGER NOT NULL,
	PRIMARY KEY("name")
);

CREATE INDEX "movements_index_0" ON "movements" ("display_order");

CREATE TABLE IF NOT EXISTS "lifts" (
	"lift_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
	"participant_id" UUID NOT NULL,
	"movement_name" VARCHAR(255) NOT NULL,
	"max_weight" DECIMAL NOT NULL CHECK (max_weight > 0),
	"equipment_setting" VARCHAR(255),
	"updated_at" TIMESTAMP,
	PRIMARY KEY("lift_id")
);

CREATE INDEX "lifts_index_0" ON "lifts" ("participant_id");
CREATE INDEX "lifts_index_1" ON "lifts" ("movement_name", "max_weight");
CREATE UNIQUE INDEX "lifts_index_2" ON "lifts" ("participant_id", "movement_name");

CREATE TABLE IF NOT EXISTS "attempts" (
	"attempt_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
	"lift_id" UUID NOT NULL,
	"attempt_number" SMALLINT NOT NULL CHECK (attempt_number BETWEEN 1 AND 3),
	"weight" DECIMAL NOT NULL CHECK (weight > 0),
	"is_successful" BOOLEAN NOT NULL,
	"passing_judges" SMALLINT CHECK (passing_judges BETWEEN 0 AND 3),
	"no_rep_reason" TEXT,
	"created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	"created_by" VARCHAR(255),
	PRIMARY KEY("attempt_id")
);

CREATE UNIQUE INDEX "attempts_index_0" ON "attempts" ("lift_id", "attempt_number");
CREATE INDEX "attempts_index_1" ON "attempts" ("is_successful");

CREATE TABLE IF NOT EXISTS "competition_groups" (
	"group_id" UUID NOT NULL DEFAULT gen_random_uuid(),
	"competition_id" UUID NOT NULL,
	"category_id" UUID NOT NULL,
	"name" VARCHAR(255) NOT NULL,
	"max_size" INTEGER CHECK (max_size > 0),
	"created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY("group_id")
);

CREATE INDEX "competition_groups_index_0" ON "competition_groups" ("competition_id");
CREATE INDEX "competition_groups_index_1" ON "competition_groups" ("category_id");
CREATE UNIQUE INDEX "competition_groups_index_2" ON "competition_groups" ("competition_id", "category_id", "name");

CREATE TABLE IF NOT EXISTS "competition_movements" (
	"competition_id" UUID NOT NULL,
	"movement_name" VARCHAR(255) NOT NULL,
	"is_required" BOOLEAN NOT NULL DEFAULT TRUE,
	"display_order" INTEGER,
	PRIMARY KEY("competition_id", "movement_name")
);

CREATE INDEX "competition_movements_index_0" ON "competition_movements" ("competition_id");

CREATE TABLE IF NOT EXISTS "socials" (
	"social_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
	"name" VARCHAR(255) NOT NULL UNIQUE,
	PRIMARY KEY("social_id")
);

CREATE TABLE IF NOT EXISTS "rulebooks" (
	"rulebook_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
	"name" VARCHAR(255) UNIQUE,
	"url" VARCHAR(500),
	PRIMARY KEY("rulebook_id")
);

CREATE TABLE IF NOT EXISTS "federations" (
	"federation_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
	"name" VARCHAR(255) NOT NULL,
	"rulebook_id" UUID,
	"country" VARCHAR(255),
	"abbreviation" VARCHAR(50),
	PRIMARY KEY("federation_id")
);

CREATE TABLE IF NOT EXISTS "athlete_socials" (
	"athlete_social_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
	"athlete_id" UUID NOT NULL,
	"social_id" UUID NOT NULL,
	"handle" VARCHAR(255) NOT NULL,
	PRIMARY KEY("athlete_social_id")
);

CREATE UNIQUE INDEX "athlete_socials_index_0" ON "athlete_socials" ("athlete_id", "social_id");
CREATE INDEX "athlete_socials_index_1" ON "athlete_socials" ("athlete_id");
CREATE UNIQUE INDEX "athlete_socials_index_2" ON "athlete_socials" ("social_id", "handle");

CREATE TABLE IF NOT EXISTS "records" (
	"record_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
	"record_type" VARCHAR(50) NOT NULL,
	"category_id" UUID NOT NULL,
	"movement_name" VARCHAR(255) NOT NULL,
	"athlete_id" UUID NOT NULL,
	"competition_id" UUID NOT NULL,
	"date_set" DATE NOT NULL,
	"weight" DECIMAL NOT NULL CHECK (weight > 0),
	"gender" VARCHAR(10) CHECK (gender IN ('M', 'F', 'MX')),
	PRIMARY KEY("record_id")
);

CREATE INDEX "records_index_0" ON "records" ("record_type");
CREATE INDEX "records_index_1" ON "records" ("athlete_id");
CREATE INDEX "records_index_2" ON "records" ("category_id", "movement_name");
CREATE UNIQUE INDEX "records_index_3" ON "records" ("record_type", "category_id", "movement_name", "gender");

-- Foreign Keys
-- Competition Groups: Categories are reference data (RESTRICT), Competitions own their groups (CASCADE)
ALTER TABLE "competition_groups"
ADD FOREIGN KEY("category_id") REFERENCES "categories"("category_id") ON UPDATE CASCADE ON DELETE RESTRICT;

ALTER TABLE "competition_groups"
ADD FOREIGN KEY("competition_id") REFERENCES "competitions"("competition_id") ON UPDATE CASCADE ON DELETE CASCADE;

-- Competition Participants: Groups own participants (CASCADE), Athletes are reference data (RESTRICT)
ALTER TABLE "competition_participants"
ADD FOREIGN KEY("group_id") REFERENCES "competition_groups"("group_id") ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE "competition_participants"
ADD FOREIGN KEY("athlete_id") REFERENCES "athletes"("athlete_id") ON UPDATE CASCADE ON DELETE RESTRICT;

-- Competition Movements: Competitions own their movement configurations (CASCADE), Movements are reference data (RESTRICT)
ALTER TABLE "competition_movements"
ADD FOREIGN KEY("competition_id") REFERENCES "competitions"("competition_id") ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE "competition_movements"
ADD FOREIGN KEY("movement_name") REFERENCES "movements"("name") ON UPDATE CASCADE ON DELETE RESTRICT;

-- Lifts: Participants own their lifts (CASCADE), Movements are reference data (RESTRICT)
ALTER TABLE "lifts"
ADD FOREIGN KEY("participant_id") REFERENCES "competition_participants"("participant_id") ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE "lifts"
ADD FOREIGN KEY("movement_name") REFERENCES "movements"("name") ON UPDATE CASCADE ON DELETE RESTRICT;

-- Attempts: Lifts own their attempts (CASCADE)
ALTER TABLE "attempts"
ADD FOREIGN KEY("lift_id") REFERENCES "lifts"("lift_id") ON UPDATE CASCADE ON DELETE CASCADE;

-- Competitions: Federations are reference data (RESTRICT)
ALTER TABLE "competitions"
ADD FOREIGN KEY("federation_id") REFERENCES "federations"("federation_id") ON UPDATE CASCADE ON DELETE RESTRICT;

-- Federations: Rulebooks are reference data, but nullable so SET NULL is more appropriate
ALTER TABLE "federations"
ADD FOREIGN KEY("rulebook_id") REFERENCES "rulebooks"("rulebook_id") ON UPDATE CASCADE ON DELETE SET NULL;

-- Athlete Socials: Athletes own their social accounts (CASCADE), Social platforms are reference data (RESTRICT)
ALTER TABLE "athlete_socials"
ADD FOREIGN KEY("athlete_id") REFERENCES "athletes"("athlete_id") ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE "athlete_socials"
ADD FOREIGN KEY("social_id") REFERENCES "socials"("social_id") ON UPDATE CASCADE ON DELETE RESTRICT;

-- Records: All references are to immutable historical data (RESTRICT to preserve data integrity)
ALTER TABLE "records"
ADD FOREIGN KEY("category_id") REFERENCES "categories"("category_id") ON UPDATE CASCADE ON DELETE RESTRICT;

ALTER TABLE "records"
ADD FOREIGN KEY("movement_name") REFERENCES "movements"("name") ON UPDATE CASCADE ON DELETE RESTRICT;

ALTER TABLE "records"
ADD FOREIGN KEY("athlete_id") REFERENCES "athletes"("athlete_id") ON UPDATE CASCADE ON DELETE RESTRICT;

ALTER TABLE "records"
ADD FOREIGN KEY("competition_id") REFERENCES "competitions"("competition_id") ON UPDATE CASCADE ON DELETE RESTRICT;
