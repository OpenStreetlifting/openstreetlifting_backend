use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct ParticipantRanking {
    participant_id: Uuid,
    category_rank: Option<i64>,
    competition_rank: Option<i64>,
}

use crate::dto::competition::{
    AthleteInfo, AttemptInfo, CategoryDetail, CategoryInfo, CompetitionDetailResponse,
    CompetitionListResponse, CreateCompetitionRequest, FederationInfo, LiftDetail, MovementInfo,
    ParticipantDetail, RankingScope,
};
use crate::error::{Result, StorageError};
use crate::models::{
    Athlete, Category, Competition, CompetitionGroup, CompetitionMovement, Federation, Lift,
};

pub struct CompetitionRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> CompetitionRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<Competition>> {
        let competitions = sqlx::query_as!(
            Competition,
            r#"
            SELECT competition_id, name, created_at, slug, status, federation_id,
                   venue, city, country, start_date, end_date, number_of_judge
            FROM competitions
            ORDER BY start_date DESC, created_at DESC
            "#
        )
        .fetch_all(self.pool)
        .await?;

        Ok(competitions)
    }

    pub async fn list_with_details(&self) -> Result<Vec<CompetitionListResponse>> {
        let competitions = self.list().await?;
        let mut results = Vec::with_capacity(competitions.len());

        for comp in competitions {
            let federation = sqlx::query_as!(
                Federation,
                "SELECT federation_id, name, rulebook_id, country, abbreviation
                 FROM federations
                 WHERE federation_id = $1",
                comp.federation_id
            )
            .fetch_one(self.pool)
            .await?;

            let movements = sqlx::query_as!(
                CompetitionMovement,
                "SELECT competition_id, movement_name, is_required, display_order
                 FROM competition_movements
                 WHERE competition_id = $1
                 ORDER BY display_order",
                comp.competition_id
            )
            .fetch_all(self.pool)
            .await?;

            results.push(CompetitionListResponse {
                competition_id: comp.competition_id,
                name: comp.name,
                created_at: comp.created_at,
                slug: comp.slug,
                status: comp.status,
                venue: comp.venue,
                city: comp.city,
                country: comp.country,
                start_date: comp.start_date,
                end_date: comp.end_date,
                federation: FederationInfo {
                    federation_id: federation.federation_id,
                    name: federation.name,
                    abbreviation: federation.abbreviation,
                    country: federation.country,
                },
                movements: movements
                    .into_iter()
                    .map(|m| MovementInfo {
                        movement_name: m.movement_name,
                        is_required: m.is_required,
                        display_order: m.display_order,
                    })
                    .collect(),
            });
        }

        Ok(results)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Competition> {
        let competition = sqlx::query_as!(
            Competition,
            r#"
            SELECT competition_id, name, created_at, slug, status, federation_id,
                   venue, city, country, start_date, end_date, number_of_judge
            FROM competitions
            WHERE competition_id = $1
            "#,
            id
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(competition)
    }

    /// Get a competition by slug
    pub async fn find_by_slug(&self, slug: &str) -> Result<Competition> {
        let competition = sqlx::query_as!(
            Competition,
            r#"
            SELECT competition_id, name, created_at, slug, status, federation_id,
                   venue, city, country, start_date, end_date, number_of_judge
            FROM competitions
            WHERE slug = $1
            "#,
            slug
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(competition)
    }

    pub async fn find_by_slug_detailed(
        &self,
        slug: &str,
        ranking_scope: RankingScope,
    ) -> Result<CompetitionDetailResponse> {
        let competition = self.find_by_slug(slug).await?;
        self.get_detailed_competition(competition, ranking_scope)
            .await
    }

    pub async fn find_by_id_detailed(
        &self,
        id: Uuid,
        ranking_scope: RankingScope,
    ) -> Result<CompetitionDetailResponse> {
        let competition = self.find_by_id(id).await?;
        self.get_detailed_competition(competition, ranking_scope)
            .await
    }

    /// Compute category and competition rankings for all participants in a competition
    async fn compute_rankings(
        &self,
        competition_id: Uuid,
        ranking_scope: &RankingScope,
    ) -> Result<HashMap<Uuid, ParticipantRanking>> {
        match ranking_scope {
            RankingScope::Group => {
                // No additional ranking needed, will use stored rank
                Ok(HashMap::new())
            }
            RankingScope::Category => {
                // Compute category rankings only
                let rankings = sqlx::query_as::<_, ParticipantRanking>(
                    r#"
                    WITH participant_totals AS (
                        SELECT
                            cp.participant_id,
                            cg.category_id,
                            cp.bodyweight,
                            cp.ris_score,
                            COALESCE(SUM(l.max_weight), 0) as total
                        FROM competition_participants cp
                        JOIN competition_groups cg ON cp.group_id = cg.group_id
                        LEFT JOIN lifts l ON l.participant_id = cp.participant_id
                        WHERE cg.competition_id = $1
                        GROUP BY cp.participant_id, cg.category_id, cp.bodyweight, cp.ris_score
                    )
                    SELECT
                        participant_id,
                        ROW_NUMBER() OVER (
                            PARTITION BY category_id
                            ORDER BY
                                CASE WHEN total = 0 THEN 1 ELSE 0 END,
                                total DESC,
                                bodyweight ASC NULLS LAST
                        )::bigint as category_rank,
                        NULL::bigint as competition_rank
                    FROM participant_totals
                    "#,
                )
                .bind(competition_id)
                .fetch_all(self.pool)
                .await?;

                Ok(rankings
                    .into_iter()
                    .map(|r| (r.participant_id, r))
                    .collect())
            }
            RankingScope::Competition => {
                // Compute both category and competition rankings
                let rankings = sqlx::query_as::<_, ParticipantRanking>(
                    r#"
                    WITH participant_totals AS (
                        SELECT
                            cp.participant_id,
                            cg.category_id,
                            cp.bodyweight,
                            cp.ris_score,
                            COALESCE(SUM(l.max_weight), 0) as total
                        FROM competition_participants cp
                        JOIN competition_groups cg ON cp.group_id = cg.group_id
                        LEFT JOIN lifts l ON l.participant_id = cp.participant_id
                        WHERE cg.competition_id = $1
                        GROUP BY cp.participant_id, cg.category_id, cp.bodyweight, cp.ris_score
                    )
                    SELECT
                        participant_id,
                        ROW_NUMBER() OVER (
                            PARTITION BY category_id
                            ORDER BY
                                CASE WHEN total = 0 THEN 1 ELSE 0 END,
                                total DESC,
                                bodyweight ASC NULLS LAST
                        )::bigint as category_rank,
                        ROW_NUMBER() OVER (
                            ORDER BY
                                CASE WHEN ris_score IS NULL OR ris_score = 0 THEN 1 ELSE 0 END,
                                ris_score DESC NULLS LAST
                        )::bigint as competition_rank
                    FROM participant_totals
                    "#,
                )
                .bind(competition_id)
                .fetch_all(self.pool)
                .await?;

                Ok(rankings
                    .into_iter()
                    .map(|r| (r.participant_id, r))
                    .collect())
            }
        }
    }

    async fn get_detailed_competition(
        &self,
        competition: Competition,
        ranking_scope: RankingScope,
    ) -> Result<CompetitionDetailResponse> {
        // Compute rankings for all participants in the competition
        let ranking_map = self
            .compute_rankings(competition.competition_id, &ranking_scope)
            .await?;

        let federation = sqlx::query_as!(
            Federation,
            "SELECT federation_id, name, rulebook_id, country, abbreviation
             FROM federations
             WHERE federation_id = $1",
            competition.federation_id
        )
        .fetch_one(self.pool)
        .await?;

        let groups = sqlx::query_as!(
            CompetitionGroup,
            "SELECT group_id, competition_id, category_id, name, max_size, created_at
             FROM competition_groups
             WHERE competition_id = $1",
            competition.competition_id
        )
        .fetch_all(self.pool)
        .await?;

        let mut category_map: HashMap<Uuid, (Category, Vec<ParticipantDetail>)> = HashMap::new();

        for group in groups {
            if let std::collections::hash_map::Entry::Vacant(e) =
                category_map.entry(group.category_id)
            {
                let category = sqlx::query_as!(
                    Category,
                    "SELECT category_id, name, gender, weight_class_min, weight_class_max
                     FROM categories
                     WHERE category_id = $1",
                    group.category_id
                )
                .fetch_one(self.pool)
                .await?;
                e.insert((category, Vec::new()));
            }
            let participants = sqlx::query!(
                "SELECT participant_id, group_id, athlete_id, bodyweight, rank, is_disqualified,
                        created_at, disqualified_reason, ris_score
                 FROM competition_participants
                 WHERE group_id = $1
                 ORDER BY rank NULLS LAST",
                group.group_id
            )
            .fetch_all(self.pool)
            .await?;

            for participant in participants {
                let athlete = sqlx::query_as!(
                    Athlete,
                    r#"SELECT athlete_id, first_name, last_name, gender, created_at,
                            nationality, country, profile_picture_url, slug,
                            COALESCE(slug_history, '[]'::jsonb) as "slug_history!: sqlx::types::Json<Vec<String>>"
                     FROM athletes
                     WHERE athlete_id = $1"#,
                    participant.athlete_id
                )
                .fetch_one(self.pool)
                .await?;

                let lifts = sqlx::query_as!(
                    Lift,
                    "SELECT lift_id, participant_id, movement_name, max_weight,
                            equipment_setting, updated_at
                     FROM lifts
                     WHERE participant_id IN (
                         SELECT participant_id FROM competition_participants
                         WHERE group_id = $1 AND athlete_id = $2
                     )",
                    group.group_id,
                    participant.athlete_id
                )
                .fetch_all(self.pool)
                .await?;

                let mut lift_details = Vec::with_capacity(lifts.len());
                let mut total = Decimal::ZERO;

                for lift in lifts {
                    let attempts = sqlx::query!(
                        "SELECT attempt_number, weight, is_successful, passing_judges, no_rep_reason
                         FROM attempts
                         WHERE lift_id = $1
                         ORDER BY attempt_number",
                        lift.lift_id
                    )
                    .fetch_all(self.pool)
                    .await?;

                    total += lift.max_weight;

                    lift_details.push(LiftDetail {
                        movement_name: lift.movement_name.clone(),
                        best_weight: lift.max_weight,
                        attempts: attempts
                            .into_iter()
                            .map(|a| AttemptInfo {
                                attempt_number: a.attempt_number,
                                weight: a.weight,
                                is_successful: a.is_successful,
                                passing_judges: a.passing_judges,
                                no_rep_reason: a.no_rep_reason,
                            })
                            .collect(),
                    });
                }

                // Get computed rankings for this participant
                let rankings = ranking_map.get(&participant.participant_id);
                let category_rank = rankings.and_then(|r| r.category_rank.map(|v| v as i32));
                let competition_rank = rankings.and_then(|r| r.competition_rank.map(|v| v as i32));

                let participant_detail = ParticipantDetail {
                    athlete: AthleteInfo {
                        athlete_id: athlete.athlete_id,
                        first_name: athlete.first_name,
                        last_name: athlete.last_name,
                        gender: athlete.gender,
                        nationality: athlete.nationality,
                        country: athlete.country,
                        slug: athlete.slug,
                    },
                    bodyweight: participant.bodyweight,
                    rank: participant.rank,
                    category_rank,
                    competition_rank,
                    ris_score: participant.ris_score,
                    is_disqualified: participant.is_disqualified,
                    disqualified_reason: participant.disqualified_reason.clone(),
                    lifts: lift_details,
                    total,
                };

                if let Some((_, participants)) = category_map.get_mut(&group.category_id) {
                    participants.push(participant_detail);
                }
            }
        }

        let mut category_details: Vec<CategoryDetail> = category_map
            .into_iter()
            .map(|(_, (category, participants))| CategoryDetail {
                category: CategoryInfo {
                    category_id: category.category_id,
                    name: category.name,
                    gender: category.gender,
                    weight_class_min: category.weight_class_min,
                    weight_class_max: category.weight_class_max,
                },
                participants,
            })
            .collect();

        category_details.sort_by(|a, b| a.category.name.cmp(&b.category.name));

        Ok(CompetitionDetailResponse {
            competition_id: competition.competition_id,
            name: competition.name,
            slug: competition.slug,
            status: competition.status,
            venue: competition.venue,
            city: competition.city,
            country: competition.country,
            start_date: competition.start_date,
            end_date: competition.end_date,
            federation: FederationInfo {
                federation_id: federation.federation_id,
                name: federation.name,
                abbreviation: federation.abbreviation,
                country: federation.country,
            },
            categories: category_details,
        })
    }

    pub async fn create(&self, req: &CreateCompetitionRequest) -> Result<Competition> {
        let competition = sqlx::query_as!(
            Competition,
            r#"
            INSERT INTO competitions (
                name, slug, status, federation_id, venue, city, country,
                start_date, end_date, number_of_judge
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING competition_id, name, created_at, slug, status, federation_id,
                      venue, city, country, start_date, end_date, number_of_judge
            "#,
            req.name,
            req.slug,
            req.status,
            req.federation_id,
            req.venue,
            req.city,
            req.country,
            req.start_date,
            req.end_date,
            req.number_of_judge
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| {
            // Handle unique constraint violations for slug
            if let sqlx::Error::Database(ref db_err) = e
                && db_err.code().as_deref() == Some("23505")
            {
                return StorageError::ConstraintViolation("Slug already exists".to_string());
            }
            StorageError::from(e)
        })?;

        Ok(competition)
    }

    pub async fn update(
        &self,
        id: Uuid,
        existing: &Competition,
        req: &crate::dto::competition::UpdateCompetitionRequest,
    ) -> Result<Competition> {
        let name = req.name.as_ref().unwrap_or(&existing.name);
        let slug = req.slug.as_ref().unwrap_or(&existing.slug);
        let status = req.status.as_ref().unwrap_or(&existing.status);
        let federation_id = req.federation_id.unwrap_or(existing.federation_id);
        let venue = req.venue.as_ref().or(existing.venue.as_ref());
        let city = req.city.as_ref().or(existing.city.as_ref());
        let country = req.country.as_ref().or(existing.country.as_ref());
        let start_date = req.start_date.or(existing.start_date);
        let end_date = req.end_date.or(existing.end_date);
        let number_of_judge = req.number_of_judge.or(existing.number_of_judge);

        let competition = sqlx::query_as!(
            Competition,
            r#"
            UPDATE competitions
            SET
                name = $2,
                slug = $3,
                status = $4,
                federation_id = $5,
                venue = $6,
                city = $7,
                country = $8,
                start_date = $9,
                end_date = $10,
                number_of_judge = $11
            WHERE competition_id = $1
            RETURNING competition_id, name, created_at, slug, status, federation_id,
                      venue, city, country, start_date, end_date, number_of_judge
            "#,
            id,
            name,
            slug,
            status,
            federation_id,
            venue,
            city,
            country,
            start_date,
            end_date,
            number_of_judge
        )
        .fetch_optional(self.pool)
        .await?
        .ok_or(StorageError::NotFound)?;

        Ok(competition)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM competitions
            WHERE competition_id = $1
            "#,
            id
        )
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound);
        }

        Ok(())
    }
}
