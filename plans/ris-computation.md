# RIS Computation Implementation Plan

## Overview

Currently, the OpenStreetLifting backend imports RIS (Relative Index for Streetlifting) scores directly from competition imports (e.g., LiftControl). This approach has several problems:

1. **Incorrect Formula Risk**: Imported competitions may have computed RIS using outdated or incorrect formulas
2. **No Historical Tracking**: Cannot show how an athlete's performance would have been rated under different years' formulas
3. **Data Inconsistency**: Different competitions may use different formula versions without documentation
4. **No Recalculation**: Cannot recompute RIS scores when formula corrections are published

## Solution

Compute RIS scores internally using year-specific formulas, storing both current and historical RIS scores for each competition result.

---

## 1. Database Schema Changes

### 1.1 New Table: `ris_formula_versions`

This table stores the formula constants for each year.

```sql
CREATE TABLE IF NOT EXISTS "ris_formula_versions" (
    "formula_id" UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    "year" INTEGER NOT NULL UNIQUE,
    "effective_from" DATE NOT NULL,
    "effective_until" DATE,
    "is_current" BOOLEAN NOT NULL DEFAULT FALSE,

    -- Men's constants
    "men_a" DECIMAL(10,5) NOT NULL,
    "men_k" DECIMAL(10,5) NOT NULL,
    "men_b" DECIMAL(10,5) NOT NULL,
    "men_v" DECIMAL(10,5) NOT NULL,
    "men_q" DECIMAL(10,5) NOT NULL,

    -- Women's constants
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
```

**Rationale**:

- Stores all historical formula versions
- `is_current` flag for quick lookup of active formula
- `effective_from`/`effective_until` dates for determining which formula to use for a competition
- Gender-specific constants as described in the RIS specification

### 1.2 New Table: `ris_scores_history`

This table stores computed RIS scores with full version history.

```sql
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
```

**Rationale**:

- Links to `competition_participants` for the athlete's competition performance
- Links to `ris_formula_versions` to track which formula was used
- Stores the inputs (bodyweight, total) alongside the computed score for audit trail
- UNIQUE constraint on (participant_id, formula_id) ensures one score per formula version per participant

### 1.3 Modify Existing Table: `competition_participants`

**Decision Required**: Keep or remove the `ris_score` column?

**Option A - Keep with Computed Column (RECOMMENDED)**:

```sql
-- Keep ris_score as a cached/denormalized field for query performance
-- Will be automatically updated via trigger or application code
-- Represents the "current" RIS formula score
```

**Option B - Remove Column**:

```sql
ALTER TABLE "competition_participants" DROP COLUMN "ris_score";
```

**Recommendation**: Keep `ris_score` as a cached field representing the current/latest formula score for:

- Query performance (avoid JOINs for common queries)
- Backward compatibility with existing API responses
- Quick sorting/filtering in leaderboards

The field will be computed/updated via application code when:

- A new participant is added
- A participant's total or bodyweight changes
- A new "current" RIS formula is published

---

## 2. Data Migration Plan

### 2.1 Migration File: `add_ris_computation_tables.sql`

**Steps**:

1. Create `ris_formula_versions` table
2. Create `ris_scores_history` table
3. Insert 2025 RIS formula constants:

   ```sql
   INSERT INTO ris_formula_versions (
       year, effective_from, is_current,
       men_a, men_k, men_b, men_v, men_q,
       women_a, women_k, women_b, women_v, women_q,
       notes
   ) VALUES (
       2025, '2025-01-01', TRUE,
       338, 549, 0.11354, 74.777, 0.53096,
       164, 270, 0.13776, 57.855, 0.37089,
       'RIS 2025 Edition - Created by Waris Radji & Mathieu Ardoin'
   );
   ```

4. For existing data in `competition_participants`, compute and populate historical RIS scores (via a separate data migration script or manual process)

### 2.2 Backfill Strategy

**Option 1 - Lazy Computation**:

- Leave existing `ris_score` in `competition_participants` as-is
- Compute historical scores on-demand when requested via API
- Cache computed scores in `ris_scores_history`

**Option 2 - Batch Backfill**:

- Create a batch script/command to iterate all `competition_participants`
- Compute RIS for all participants using the 2025 formula (and any historical formulas)
- Populate `ris_scores_history` table

**Recommendation**: Use Option 2 (Batch Backfill) to ensure data consistency and avoid computation delays during API calls.

---

## 3. Service Layer Changes

### 3.1 New Module: `crates/storage/src/services/ris_computation.rs`

**Responsibilities**:

- Compute RIS score given (bodyweight, total, gender, formula_id)
- Retrieve formula constants by year or formula_id
- Determine which formula to use based on competition date
- Provide batch computation utilities

**Key Functions**:

```rust
/// Compute RIS score using specific formula version
pub fn compute_ris(
    bodyweight: Decimal,
    total: Decimal,
    gender: &str,
    formula: &RisFormulaVersion,
) -> Result<Decimal>;

/// Get the appropriate formula for a competition date
pub async fn get_formula_for_date(
    pool: &PgPool,
    competition_date: NaiveDate,
) -> Result<RisFormulaVersion>;

/// Get the current active formula
pub async fn get_current_formula(
    pool: &PgPool,
) -> Result<RisFormulaVersion>;

/// Compute and store RIS for a participant using current formula
pub async fn compute_and_store_ris(
    pool: &PgPool,
    participant_id: Uuid,
    bodyweight: Decimal,
    total: Decimal,
    gender: &str,
) -> Result<Decimal>;

/// Compute historical RIS scores for a participant (all formula versions)
pub async fn compute_historical_ris(
    pool: &PgPool,
    participant_id: Uuid,
    bodyweight: Decimal,
    total: Decimal,
    gender: &str,
) -> Result<Vec<RisScoreHistory>>;

/// Batch recompute RIS for all participants (useful for migrations)
pub async fn recompute_all_ris(
    pool: &PgPool,
    formula_id: Option<Uuid>,
) -> Result<u64>;
```

**Formula Implementation**:

```rust
// RIS = Total × 100 / (A + (K - A) / (1 + Q · e^(-B · (BW - v))))

fn compute_ris_formula(
    total: Decimal,
    bodyweight: Decimal,
    a: Decimal,
    k: Decimal,
    b: Decimal,
    v: Decimal,
    q: Decimal,
) -> Decimal {
    let bw_minus_v = bodyweight - v;
    let exp_term = (-b * bw_minus_v).exp();
    let denominator_fraction = (k - a) / (Decimal::ONE + q * exp_term);
    let denominator = a + denominator_fraction;

    (total * Decimal::from(100)) / denominator
}
```

### 3.2 New Models

**Model: `RisFormulaVersion`** (`crates/storage/src/models/ris_formula.rs`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RisFormulaVersion {
    pub formula_id: Uuid,
    pub year: i32,
    pub effective_from: NaiveDate,
    pub effective_until: Option<NaiveDate>,
    pub is_current: bool,

    pub men_a: Decimal,
    pub men_k: Decimal,
    pub men_b: Decimal,
    pub men_v: Decimal,
    pub men_q: Decimal,

    pub women_a: Decimal,
    pub women_k: Decimal,
    pub women_b: Decimal,
    pub women_v: Decimal,
    pub women_q: Decimal,

    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}
```

**Model: `RisScoreHistory`** (`crates/storage/src/models/ris_score.rs`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RisScoreHistory {
    pub ris_score_id: Uuid,
    pub participant_id: Uuid,
    pub formula_id: Uuid,
    pub ris_score: Decimal,
    pub bodyweight: Decimal,
    pub total_weight: Decimal,
    pub computed_at: NaiveDateTime,
}
```

---

## 4. Repository Layer Changes

### 4.1 New Repository: `crates/storage/src/repository/ris.rs`

**Methods**:

```rust
/// Get formula by ID
async fn get_formula_by_id(pool: &PgPool, formula_id: Uuid) -> Result<Option<RisFormulaVersion>>;

/// Get formula by year
async fn get_formula_by_year(pool: &PgPool, year: i32) -> Result<Option<RisFormulaVersion>>;

/// Get current formula
async fn get_current_formula(pool: &PgPool) -> Result<Option<RisFormulaVersion>>;

/// Get formula effective on a specific date
async fn get_formula_for_date(pool: &PgPool, date: NaiveDate) -> Result<Option<RisFormulaVersion>>;

/// List all formula versions
async fn list_all_formulas(pool: &PgPool) -> Result<Vec<RisFormulaVersion>>;

/// Insert RIS score into history
async fn insert_ris_score(
    pool: &PgPool,
    participant_id: Uuid,
    formula_id: Uuid,
    ris_score: Decimal,
    bodyweight: Decimal,
    total_weight: Decimal,
) -> Result<RisScoreHistory>;

/// Get RIS score history for a participant
async fn get_participant_ris_history(
    pool: &PgPool,
    participant_id: Uuid,
) -> Result<Vec<RisScoreHistory>>;

/// Get specific RIS score for a participant using a formula
async fn get_participant_ris_for_formula(
    pool: &PgPool,
    participant_id: Uuid,
    formula_id: Uuid,
) -> Result<Option<RisScoreHistory>>;

/// Update current RIS score in competition_participants
async fn update_participant_current_ris(
    pool: &PgPool,
    participant_id: Uuid,
    ris_score: Decimal,
) -> Result<()>;
```

### 4.2 Update Existing Repository: `crates/storage/src/repository/competition.rs`

**Modifications**:

- Update `get_competition_details()` to optionally include RIS history
- Ensure participant queries include current RIS from `competition_participants`

---

## 5. API/Handler Changes

### 5.1 New DTOs

**File: `crates/storage/src/dto/ris.rs`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RisScoreResponse {
    pub formula_year: i32,
    pub ris_score: Decimal,
    pub bodyweight: Decimal,
    pub total_weight: Decimal,
    pub computed_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RisHistoryResponse {
    pub participant_id: Uuid,
    pub athlete_name: String,
    pub current_ris: Decimal,
    pub historical_scores: Vec<RisScoreResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RisFormulaResponse {
    pub formula_id: Uuid,
    pub year: i32,
    pub is_current: bool,
    pub effective_from: NaiveDate,
    pub effective_until: Option<NaiveDate>,
    pub constants: RisConstants,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RisConstants {
    pub men: GenderConstants,
    pub women: GenderConstants,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GenderConstants {
    pub a: Decimal,
    pub k: Decimal,
    pub b: Decimal,
    pub v: Decimal,
    pub q: Decimal,
}
```

### 5.2 Update Existing DTOs

**File: `crates/storage/src/dto/competition.rs`**

Modify `ParticipantDetail`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParticipantDetail {
    pub athlete: AthleteInfo,
    pub bodyweight: Option<rust_decimal::Decimal>,
    pub rank: Option<i32>,
    pub ris_score: Option<rust_decimal::Decimal>,  // Current RIS
    pub ris_history: Option<Vec<RisScoreResponse>>, // NEW: Historical RIS scores
    pub is_disqualified: bool,
    pub disqualified_reason: Option<String>,
    pub lifts: Vec<LiftDetail>,
    pub total: rust_decimal::Decimal,
}
```

**File: `crates/storage/src/dto/athlete.rs`**

Modify `AthleteCompetitionSummary`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AthleteCompetitionSummary {
    pub competition_id: Uuid,
    pub competition_name: String,
    pub competition_slug: String,
    pub competition_date: Option<chrono::NaiveDate>,
    pub category_name: String,
    pub rank: Option<i32>,
    pub total: rust_decimal::Decimal,
    pub ris_score: Option<rust_decimal::Decimal>,  // Current RIS
    pub ris_history: Option<Vec<RisScoreResponse>>, // NEW: Optional historical RIS
    pub is_disqualified: bool,
}
```

### 5.3 New API Endpoints

**File: `crates/web/src/handlers/ris.rs`**

```rust
/// GET /api/ris/formulas
/// List all RIS formula versions
async fn list_ris_formulas(pool: web::Data<PgPool>) -> Result<HttpResponse>;

/// GET /api/ris/formulas/current
/// Get the current active RIS formula
async fn get_current_formula(pool: web::Data<PgPool>) -> Result<HttpResponse>;

/// GET /api/ris/formulas/{year}
/// Get RIS formula for a specific year
async fn get_formula_by_year(
    pool: web::Data<PgPool>,
    year: web::Path<i32>,
) -> Result<HttpResponse>;

/// GET /api/participants/{participant_id}/ris-history
/// Get all RIS scores (historical) for a specific participant
async fn get_participant_ris_history(
    pool: web::Data<PgPool>,
    participant_id: web::Path<Uuid>,
) -> Result<HttpResponse>;

/// POST /api/ris/compute
/// Compute RIS score for given parameters (utility endpoint)
#[derive(Deserialize, Validate)]
struct ComputeRisRequest {
    bodyweight: Decimal,
    total: Decimal,
    gender: String, // "M" or "F"
    formula_year: Option<i32>, // Use current if not specified
}
async fn compute_ris(
    pool: web::Data<PgPool>,
    payload: web::Json<ComputeRisRequest>,
) -> Result<HttpResponse>;

/// POST /api/admin/ris/recompute-all
/// Recompute all RIS scores (admin only, requires authentication)
async fn recompute_all_ris(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse>;
```

**File: `crates/web/src/routes/ris.rs`**

```rust
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/ris")
            .route("/formulas", web::get().to(list_ris_formulas))
            .route("/formulas/current", web::get().to(get_current_formula))
            .route("/formulas/{year}", web::get().to(get_formula_by_year))
            .route("/compute", web::post().to(compute_ris))
    )
    .service(
        web::scope("/participants")
            .route("/{participant_id}/ris-history", web::get().to(get_participant_ris_history))
    )
    .service(
        web::scope("/admin/ris")
            .route("/recompute-all", web::post().to(recompute_all_ris))
    );
}
```

Update `crates/web/src/routes/mod.rs` to include the new routes.

---

## 6. Importer Changes

### 6.1 Modify LiftControl Transformer

**File: `crates/importer/src/sources/liftcontrol/transformer.rs`**

**Changes in `import_athlete_performance()` method (around line 235-275)**:

**Current Code**:

```rust
let ris_score = convert_weight(athlete_data.ris);

sqlx::query!(
    r#"
    INSERT INTO competition_participants
        (group_id, athlete_id, bodyweight, rank, is_disqualified, disqualified_reason, ris_score)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    ON CONFLICT (group_id, athlete_id)
    DO UPDATE SET
        bodyweight = EXCLUDED.bodyweight,
        rank = EXCLUDED.rank,
        is_disqualified = EXCLUDED.is_disqualified,
        disqualified_reason = EXCLUDED.disqualified_reason,
        ris_score = EXCLUDED.ris_score
    "#,
    group_id,
    athlete_id,
    bodyweight,
    rank,
    athlete_data.athlete_info.is_out,
    athlete_data.athlete_info.reason_out,
    ris_score  // ← REMOVE THIS
)
```

**New Code**:

```rust
// DO NOT import RIS from external source - we will compute it ourselves
// let ris_score = convert_weight(athlete_data.ris); ← REMOVE

sqlx::query!(
    r#"
    INSERT INTO competition_participants
        (group_id, athlete_id, bodyweight, rank, is_disqualified, disqualified_reason)
    VALUES ($1, $2, $3, $4, $5, $6)
    ON CONFLICT (group_id, athlete_id)
    DO UPDATE SET
        bodyweight = EXCLUDED.bodyweight,
        rank = EXCLUDED.rank,
        is_disqualified = EXCLUDED.is_disqualified,
        disqualified_reason = EXCLUDED.disqualified_reason
    "#,
    group_id,
    athlete_id,
    bodyweight,
    rank,
    athlete_data.athlete_info.is_out,
    athlete_data.athlete_info.reason_out
)
```

**Add RIS Computation After Import**:

At the end of `import_competition()` method (after line 58, before `tx.commit()`):

```rust
// After all participants are imported, compute RIS scores
info!("Computing RIS scores for all participants...");
self.compute_ris_for_competition(competition_id, &mut tx).await?;

tx.commit().await?;
Ok(())
```

**New Method in `LiftControlTransformer`**:

```rust
async fn compute_ris_for_competition(
    &self,
    competition_id: Uuid,
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<()> {
    // 1. Get competition date to determine which RIS formula to use
    let competition_date = self.metadata.start_date;

    // 2. Get appropriate RIS formula
    let formula = storage::services::ris_computation::get_formula_for_date(&self.pool, competition_date)
        .await?
        .ok_or_else(|| ImporterError::TransformationError(
            format!("No RIS formula available for date {}", competition_date)
        ))?;

    // 3. Get all participants for this competition with their totals
    let participants = sqlx::query!(
        r#"
        SELECT
            cp.participant_id,
            cp.athlete_id,
            cp.bodyweight,
            a.gender,
            COALESCE(SUM(l.max_weight), 0) as "total!: Decimal"
        FROM competition_participants cp
        INNER JOIN competition_groups cg ON cp.group_id = cg.group_id
        INNER JOIN athletes a ON cp.athlete_id = a.athlete_id
        LEFT JOIN lifts l ON l.participant_id = cp.participant_id
        WHERE cg.competition_id = $1
        GROUP BY cp.participant_id, cp.athlete_id, cp.bodyweight, a.gender
        "#,
        competition_id
    )
    .fetch_all(&mut **tx)
    .await?;

    // 4. Compute and store RIS for each participant
    for participant in participants {
        if let Some(bodyweight) = participant.bodyweight {
            let ris_score = storage::services::ris_computation::compute_ris(
                bodyweight,
                participant.total,
                &participant.gender,
                &formula,
            )?;

            // Update current RIS in competition_participants
            sqlx::query!(
                r#"
                UPDATE competition_participants
                SET ris_score = $1
                WHERE participant_id = $2
                "#,
                ris_score,
                participant.participant_id
            )
            .execute(&mut **tx)
            .await?;

            // Store in history
            sqlx::query!(
                r#"
                INSERT INTO ris_scores_history (participant_id, formula_id, ris_score, bodyweight, total_weight)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (participant_id, formula_id)
                DO UPDATE SET
                    ris_score = EXCLUDED.ris_score,
                    bodyweight = EXCLUDED.bodyweight,
                    total_weight = EXCLUDED.total_weight,
                    computed_at = CURRENT_TIMESTAMP
                "#,
                participant.participant_id,
                formula.formula_id,
                ris_score,
                bodyweight,
                participant.total
            )
            .execute(&mut **tx)
            .await?;
        }
    }

    info!("Computed RIS for {} participants", participants.len());
    Ok(())
}
```

### 6.2 Remove RIS Field from LiftControl Models (Optional)

**File: `crates/importer/src/sources/liftcontrol/models.rs`**

Since we're no longer using the imported RIS value, you can optionally remove it from the model or keep it for reference:

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AthleteData {
    #[serde(rename = "athleteInfo")]
    pub athlete_info: AthleteInfo,
    pub results: HashMap<String, MovementResults>,
    pub total: f64,
    // Keep but don't use, or remove entirely:
    // #[serde(rename = "RIS")]
    // pub ris: f64,
    pub rank: AthleteRank,
}
```

---

## 7. Implementation Order

### Phase 1: Database Foundation

1. Create migration file for new tables
2. Add 2025 RIS formula to `ris_formula_versions`
3. Run migration on development database
4. Test database schema

### Phase 2: Core RIS Computation

5. Create `RisFormulaVersion` model
6. Create `RisScoreHistory` model
7. Create RIS repository layer (`crates/storage/src/repository/ris.rs`)
8. Create RIS computation service (`crates/storage/src/services/ris_computation.rs`)
9. Write unit tests for RIS computation formula
10. Write integration tests for RIS service

### Phase 3: Importer Updates

11. Modify `liftcontrol/transformer.rs` to remove RIS import
12. Add RIS computation to importer workflow
13. Test import with sample LiftControl data
14. Verify RIS scores are computed correctly

### Phase 4: API Layer

15. Create RIS DTOs (`crates/storage/src/dto/ris.rs`)
16. Update existing DTOs (competition, athlete) to include RIS history
17. Create RIS handlers (`crates/web/src/handlers/ris.rs`)
18. Add RIS routes (`crates/web/src/routes/ris.rs`)
19. Update OpenAPI documentation

### Phase 5: Data Migration & Backfill

20. Create batch script to recompute existing RIS scores
21. Run backfill on staging database
22. Verify data integrity
23. Run backfill on production database

### Phase 6: Testing & Documentation

24. End-to-end testing with real competition data
25. Performance testing for RIS computation at scale
26. Update API documentation
27. Create admin guide for managing RIS formulas

---

## 8. Testing Strategy

### Unit Tests

- RIS formula computation with known inputs/outputs
- Formula selection based on competition date
- Edge cases: zero bodyweight, zero total, missing data

### Integration Tests

- Import competition and verify RIS computation
- Retrieve historical RIS scores for participant
- Update formula and recompute all RIS scores
- API endpoints return correct RIS data

### Test Data

Use the 2025 RIS constants to verify computation:

- Men: bodyweight=75kg, total=450kg → Expected RIS ≈ 82.28
- Women: bodyweight=60kg, total=250kg → Expected RIS ≈ 93.40

---

## 9. Future Enhancements

### 9.1 Admin Interface for Formula Management

- Web UI to add new RIS formula versions
- Ability to mark a formula as "current"
- Batch recomputation trigger

### 9.2 RIS Ranking/Leaderboard

- Global RIS rankings across all competitions
- Filter by year, gender, weight class
- Compare athlete RIS over time

### 9.3 Historical RIS Visualization

- Chart showing how an athlete's performance would be rated under different formulas
- Comparison of RIS formula impact across athletes

### 9.4 Automatic Formula Updates

- Integration with RIS official repository/API (if available)
- Automatic notification when new formula version is published

---

## 10. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Incorrect RIS formula implementation | High | Unit tests with known values, cross-validation with official calculator |
| Performance impact of computing RIS for large competitions | Medium | Batch processing, async computation, caching in `competition_participants` |
| Data loss during migration | High | Thorough backups, test on staging first, rollback plan |
| Inconsistent historical data | Medium | Clear documentation on when formula versions were applied |
| Breaking API changes | Medium | Maintain backward compatibility, add new fields as optional |

---

## 11. Success Criteria

- [ ] All existing competitions have computed RIS scores using 2025 formula
- [ ] No imported RIS scores are used from external sources
- [ ] API returns both current and historical RIS scores
- [ ] RIS computation performance is acceptable (<1s for 100 participants)
- [ ] Admin can add new formula versions without code changes
- [ ] All tests pass with >90% code coverage for RIS module

---

## Questions & Decisions Needed

1. **Should we keep the `ris_score` column in `competition_participants`?**
   - Recommendation: YES, for performance and backward compatibility

2. **Should we compute historical RIS on-demand or pre-compute?**
   - Recommendation: Pre-compute during import and via batch script for existing data

3. **Do we need versioning for competitions (which formula was "active" when the competition happened)?**
   - Recommendation: YES, use `effective_from`/`effective_until` dates in `ris_formula_versions`

4. **Should the RIS computation be synchronous or asynchronous?**
   - Recommendation: Synchronous during import (within transaction), async for batch recomputation

5. **Do we need to store formula constants with higher precision than 5 decimal places?**
   - Recommendation: 5 decimal places should be sufficient based on 2025 constants

6. **Should we allow multiple "current" formulas (e.g., different regions/federations)?**
   - Recommendation: Not in initial implementation, but schema supports this (just remove UNIQUE constraint on `is_current`)

---

## Appendix: RIS Formula Reference

**Formula**:

```
RIS = Total × 100 / (A + (K - A) / (1 + Q · e^(-B · (BW - v))))
```

**2025 Constants**:

| Constant | Men   | Women  |
|----------|-------|--------|
| A        | 338   | 164    |
| K        | 549   | 270    |
| B        | 0.11354 | 0.13776 |
| v        | 74.777 | 57.855 |
| Q        | 0.53096 | 0.37089 |

**Example Calculation** (Men, BW=75kg, Total=450kg):

```
RIS = 450 × 100 / (338 + (549 - 338) / (1 + 0.53096 · e^(-0.11354 · (75 - 74.777))))
    = 45000 / (338 + 211 / (1 + 0.53096 · e^(-0.025)))
    = 45000 / (338 + 211 / (1 + 0.53096 · 0.9753))
    = 45000 / (338 + 211 / 1.5180)
    = 45000 / (338 + 138.97)
    = 45000 / 476.97
    ≈ 94.35
```
