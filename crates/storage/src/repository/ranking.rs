use rust_decimal::Decimal;
use sqlx::{PgPool, QueryBuilder, Row};

use crate::dto::ranking::{AthleteInfo, CompetitionInfo, GlobalRankingEntry, GlobalRankingFilter};
use crate::error::Result;

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

        // Get total count with filters
        let total_items = self.count_participants(filter).await?;

        // Get ranked entries with filters
        let entries = self.fetch_ranked_entries(filter, offset, limit).await?;

        Ok((entries, total_items))
    }

    async fn count_participants(&self, filter: &GlobalRankingFilter) -> Result<i64> {
        let mut query = QueryBuilder::new(
            "SELECT COUNT(DISTINCT cp.participant_id) FROM competition_participants cp \
             INNER JOIN athletes a ON cp.athlete_id = a.athlete_id \
             INNER JOIN lifts l ON cp.participant_id = l.participant_id WHERE 1=1",
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
        let sort_column = self.get_sort_column(&filter.movement);

        let mut query = QueryBuilder::new(
            "WITH movement_weights AS ( \
                SELECT \
                    cp.participant_id, \
                    a.athlete_id, \
                    a.first_name, \
                    a.last_name, \
                    a.slug, \
                    a.country, \
                    a.gender, \
                    c.competition_id, \
                    c.name as competition_name, \
                    c.start_date, \
                    COALESCE(MAX(CASE WHEN l.movement_name = 'Muscle-up' THEN l.max_weight END), 0) as muscleup, \
                    COALESCE(MAX(CASE WHEN l.movement_name = 'Pull-up' THEN l.max_weight END), 0) as pullup, \
                    COALESCE(MAX(CASE WHEN l.movement_name = 'Dips' THEN l.max_weight END), 0) as dips, \
                    COALESCE(MAX(CASE WHEN l.movement_name = 'Squat' THEN l.max_weight END), 0) as squat, \
                    COALESCE(SUM(l.max_weight), 0) as total \
                FROM competition_participants cp \
                INNER JOIN athletes a ON cp.athlete_id = a.athlete_id \
                INNER JOIN competition_groups cg ON cp.group_id = cg.group_id \
                INNER JOIN competitions c ON cg.competition_id = c.competition_id \
                INNER JOIN lifts l ON cp.participant_id = l.participant_id \
                WHERE 1=1",
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
            " GROUP BY cp.participant_id, a.athlete_id, a.first_name, a.last_name, \
                       a.slug, a.country, a.gender, c.competition_id, c.name, c.start_date \
            ), \
            ranked_movements AS ( \
                SELECT *, ROW_NUMBER() OVER (ORDER BY ",
        );
        query.push(sort_column);
        query.push(
            " DESC) as rank FROM movement_weights \
            ) \
            SELECT * FROM ranked_movements \
            ORDER BY rank \
            LIMIT ",
        );
        query.push_bind(limit);
        query.push(" OFFSET ");
        query.push_bind(offset);

        let rows = query.build().fetch_all(self.pool).await?;

        let entries = rows
            .into_iter()
            .map(|row| GlobalRankingEntry {
                rank: row.get("rank"),
                athlete: AthleteInfo {
                    athlete_id: row.get("athlete_id"),
                    first_name: row.get("first_name"),
                    last_name: row.get("last_name"),
                    slug: row.get("slug"),
                    country: row.get("country"),
                    gender: row.get("gender"),
                },
                total: decimal_to_f64(row.get("total")),
                muscleup: decimal_to_f64(row.get("muscleup")),
                pullup: decimal_to_f64(row.get("pullup")),
                dips: decimal_to_f64(row.get("dips")),
                squat: decimal_to_f64(row.get("squat")),
                competition: CompetitionInfo {
                    competition_id: row.get("competition_id"),
                    name: row.get("competition_name"),
                    date: row.get("start_date"),
                },
            })
            .collect();

        Ok(entries)
    }

    fn get_sort_column(&self, movement: &str) -> &'static str {
        match movement {
            "muscleup" => "muscleup",
            "pullup" => "pullup",
            "dips" => "dips",
            "squat" => "squat",
            _ => "total",
        }
    }
}

fn decimal_to_f64(decimal: Decimal) -> f64 {
    decimal.to_string().parse().unwrap_or(0.0)
}
