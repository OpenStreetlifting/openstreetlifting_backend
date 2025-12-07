use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool, QueryBuilder};
use uuid::Uuid;

use crate::dto::ranking::{AthleteInfo, CompetitionInfo, GlobalRankingEntry, GlobalRankingFilter};
use crate::error::Result;

#[derive(FromRow)]
struct RankingRow {
    rank: i64,
    athlete_id: Uuid,
    first_name: String,
    last_name: String,
    slug: String,
    country: String,
    gender: String,
    bodyweight: Option<Decimal>,
    competition_id: Uuid,
    competition_name: String,
    start_date: Option<NaiveDate>,
    muscleup: Decimal,
    pullup: Decimal,
    dips: Decimal,
    squat: Decimal,
    total: Decimal,
    ris_score: Option<Decimal>,
}

pub struct RankingRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> RankingRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_global_ranking(
        &self,
        filter: &GlobalRankingFilter,
    ) -> Result<(Vec<GlobalRankingEntry>, i64)> {
        let offset = filter.pagination.offset() as i64;
        let limit = filter.pagination.limit() as i64;

        let total_items = self.count_participants(filter).await?;

        let entries = self.fetch_ranked_entries(filter, offset, limit).await?;

        Ok((entries, total_items))
    }

    async fn count_participants(&self, filter: &GlobalRankingFilter) -> Result<i64> {
        let mut query = QueryBuilder::new(
            r#"
            SELECT COUNT(DISTINCT cp.participant_id)
            FROM competition_participants cp
            INNER JOIN athletes a ON cp.athlete_id = a.athlete_id
            INNER JOIN lifts l ON cp.participant_id = l.participant_id
            WHERE 1=1
            "#,
        );

        if let Some(ref gender) = filter.gender {
            query.push(" AND a.gender = ");
            query.push_bind(gender);
        }

        if let Some(ref country) = filter.country {
            query.push(" AND a.country = ");
            query.push_bind(country);
        }

        let count = query
            .build_query_scalar::<i64>()
            .fetch_one(self.pool)
            .await?;

        Ok(count)
    }

    async fn fetch_ranked_entries(
        &self,
        filter: &GlobalRankingFilter,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<GlobalRankingEntry>> {
        let sort_column = filter.movement.as_column();

        let mut query = QueryBuilder::new(
            r#"
            WITH movement_weights AS (
                SELECT
                    cp.participant_id,
                    a.athlete_id,
                    a.first_name,
                    a.last_name,
                    a.slug,
                    a.country,
                    a.gender,
                    cp.bodyweight,
                    c.competition_id,
                    c.name as competition_name,
                    c.start_date,
                    COALESCE(MAX(CASE WHEN l.movement_name = 'Muscle-up' THEN l.max_weight END), 0) as muscleup,
                    COALESCE(MAX(CASE WHEN l.movement_name = 'Pull-up' THEN l.max_weight END), 0) as pullup,
                    COALESCE(MAX(CASE WHEN l.movement_name = 'Dips' THEN l.max_weight END), 0) as dips,
                    COALESCE(MAX(CASE WHEN l.movement_name = 'Squat' THEN l.max_weight END), 0) as squat,
                    COALESCE(SUM(l.max_weight), 0) as total,
                    MAX(rsh.ris_score) as ris_score
                FROM competition_participants cp
                INNER JOIN athletes a ON cp.athlete_id = a.athlete_id
                INNER JOIN competitions c ON cp.competition_id = c.competition_id
                INNER JOIN lifts l ON cp.participant_id = l.participant_id
                LEFT JOIN ris_scores_history rsh ON rsh.participant_id = cp.participant_id
                WHERE 1=1
            "#,
        );

        if let Some(ref gender) = filter.gender {
            query.push(" AND a.gender = ");
            query.push_bind(gender);
        }

        if let Some(ref country) = filter.country {
            query.push(" AND a.country = ");
            query.push_bind(country);
        }

        query.push(
            r#"
                GROUP BY cp.participant_id, a.athlete_id, a.first_name, a.last_name,
                         a.slug, a.country, a.gender, cp.bodyweight, c.competition_id, c.name, c.start_date
            ),
            ranked_movements AS (
                SELECT *, ROW_NUMBER() OVER (ORDER BY
            "#,
        );
        query.push(sort_column);
        query.push(
            r#"
                DESC) as rank
                FROM movement_weights
            )
            SELECT * FROM ranked_movements
            ORDER BY rank
            LIMIT
            "#,
        );
        query.push_bind(limit);
        query.push(" OFFSET ");
        query.push_bind(offset);

        let rows: Vec<RankingRow> = query.build_query_as().fetch_all(self.pool).await?;

        let entries = rows
            .into_iter()
            .map(|row| GlobalRankingEntry {
                rank: row.rank,
                athlete: AthleteInfo {
                    athlete_id: row.athlete_id,
                    first_name: row.first_name,
                    last_name: row.last_name,
                    slug: row.slug,
                    country: row.country,
                    gender: row.gender,
                    bodyweight: row.bodyweight.map(decimal_to_f64),
                },
                ris: row.ris_score.map(decimal_to_f64).unwrap_or(0.0),
                total: decimal_to_f64(row.total),
                muscleup: decimal_to_f64(row.muscleup),
                pullup: decimal_to_f64(row.pullup),
                dips: decimal_to_f64(row.dips),
                squat: decimal_to_f64(row.squat),
                competition: CompetitionInfo {
                    competition_id: row.competition_id,
                    name: row.competition_name,
                    date: row.start_date,
                },
            })
            .collect();

        Ok(entries)
    }
}

fn decimal_to_f64(decimal: Decimal) -> f64 {
    decimal.to_string().parse().unwrap_or(0.0)
}
