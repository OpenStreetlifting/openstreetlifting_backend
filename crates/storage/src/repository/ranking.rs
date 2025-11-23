use rust_decimal::Decimal;
use sqlx::{PgPool, Row};

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
        self.get_all_movements_ranking(filter, offset, limit).await
    }

    async fn get_all_movements_ranking(
        &self,
        filter: &GlobalRankingFilter,
        offset: i64,
        limit: i64,
    ) -> Result<(Vec<GlobalRankingEntry>, i64)> {
        let sort_field = match filter.movement.as_str() {
            "muscleup" => "muscleup",
            "pullup" => "pullup",
            "dips" => "dips",
            "squat" => "squat",
            _ => "total",
        };

        match (&filter.gender, &filter.country) {
            (Some(gender), Some(country)) => {
                let total_items = sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(DISTINCT cp.participant_id)
                    FROM competition_participants cp
                    INNER JOIN athletes a ON cp.athlete_id = a.athlete_id
                    INNER JOIN lifts l ON cp.participant_id = l.participant_id
                    WHERE a.gender = $1 AND a.country = $2
                    "#,
                    gender,
                    country
                )
                .fetch_one(self.pool)
                .await?
                .unwrap_or(0);

                let query = format!(
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
                            c.competition_id,
                            c.name as competition_name,
                            c.start_date,
                            MAX(CASE WHEN l.movement_name = 'muscleup' THEN l.max_weight ELSE 0 END) as muscleup,
                            MAX(CASE WHEN l.movement_name = 'pullup' THEN l.max_weight ELSE 0 END) as pullup,
                            MAX(CASE WHEN l.movement_name = 'dips' THEN l.max_weight ELSE 0 END) as dips,
                            MAX(CASE WHEN l.movement_name = 'squat' THEN l.max_weight ELSE 0 END) as squat,
                            SUM(l.max_weight) as total
                        FROM competition_participants cp
                        INNER JOIN athletes a ON cp.athlete_id = a.athlete_id
                        INNER JOIN competition_groups cg ON cp.group_id = cg.group_id
                        INNER JOIN competitions c ON cg.competition_id = c.competition_id
                        INNER JOIN lifts l ON cp.participant_id = l.participant_id
                        WHERE a.gender = $1 AND a.country = $2
                        GROUP BY cp.participant_id, a.athlete_id, a.first_name, a.last_name,
                                 a.slug, a.country, a.gender, c.competition_id, c.name, c.start_date
                    )
                    SELECT
                        ROW_NUMBER() OVER (ORDER BY {} DESC) as rank,
                        athlete_id,
                        first_name,
                        last_name,
                        slug,
                        country,
                        gender,
                        competition_id,
                        competition_name,
                        start_date,
                        total,
                        muscleup,
                        pullup,
                        dips,
                        squat
                    FROM movement_weights
                    ORDER BY {} DESC
                    LIMIT $3 OFFSET $4
                    "#,
                    sort_field, sort_field
                );

                let rows = sqlx::query(&query)
                    .bind(gender)
                    .bind(country)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(self.pool)
                    .await?;

                let entries = rows
                    .into_iter()
                    .map(|row| {
                        let rank: i64 = row.get("rank");
                        GlobalRankingEntry {
                            rank: rank + offset,
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
                        }
                    })
                    .collect();

                Ok((entries, total_items))
            }
            (Some(gender), None) => {
                let total_items = sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(DISTINCT cp.participant_id)
                    FROM competition_participants cp
                    INNER JOIN athletes a ON cp.athlete_id = a.athlete_id
                    INNER JOIN lifts l ON cp.participant_id = l.participant_id
                    WHERE a.gender = $1
                    "#,
                    gender
                )
                .fetch_one(self.pool)
                .await?
                .unwrap_or(0);

                let query = format!(
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
                            c.competition_id,
                            c.name as competition_name,
                            c.start_date,
                            MAX(CASE WHEN l.movement_name = 'muscleup' THEN l.max_weight ELSE 0 END) as muscleup,
                            MAX(CASE WHEN l.movement_name = 'pullup' THEN l.max_weight ELSE 0 END) as pullup,
                            MAX(CASE WHEN l.movement_name = 'dips' THEN l.max_weight ELSE 0 END) as dips,
                            MAX(CASE WHEN l.movement_name = 'squat' THEN l.max_weight ELSE 0 END) as squat,
                            SUM(l.max_weight) as total
                        FROM competition_participants cp
                        INNER JOIN athletes a ON cp.athlete_id = a.athlete_id
                        INNER JOIN competition_groups cg ON cp.group_id = cg.group_id
                        INNER JOIN competitions c ON cg.competition_id = c.competition_id
                        INNER JOIN lifts l ON cp.participant_id = l.participant_id
                        WHERE a.gender = $1
                        GROUP BY cp.participant_id, a.athlete_id, a.first_name, a.last_name,
                                 a.slug, a.country, a.gender, c.competition_id, c.name, c.start_date
                    )
                    SELECT
                        ROW_NUMBER() OVER (ORDER BY {} DESC) as rank,
                        athlete_id,
                        first_name,
                        last_name,
                        slug,
                        country,
                        gender,
                        competition_id,
                        competition_name,
                        start_date,
                        total,
                        muscleup,
                        pullup,
                        dips,
                        squat
                    FROM movement_weights
                    ORDER BY {} DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    sort_field, sort_field
                );

                let rows = sqlx::query(&query)
                    .bind(gender)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(self.pool)
                    .await?;

                let entries = rows
                    .into_iter()
                    .map(|row| {
                        let rank: i64 = row.get("rank");
                        GlobalRankingEntry {
                            rank: rank + offset,
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
                        }
                    })
                    .collect();

                Ok((entries, total_items))
            }
            (None, Some(country)) => {
                let total_items = sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(DISTINCT cp.participant_id)
                    FROM competition_participants cp
                    INNER JOIN athletes a ON cp.athlete_id = a.athlete_id
                    INNER JOIN lifts l ON cp.participant_id = l.participant_id
                    WHERE a.country = $1
                    "#,
                    country
                )
                .fetch_one(self.pool)
                .await?
                .unwrap_or(0);

                let query = format!(
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
                            c.competition_id,
                            c.name as competition_name,
                            c.start_date,
                            MAX(CASE WHEN l.movement_name = 'muscleup' THEN l.max_weight ELSE 0 END) as muscleup,
                            MAX(CASE WHEN l.movement_name = 'pullup' THEN l.max_weight ELSE 0 END) as pullup,
                            MAX(CASE WHEN l.movement_name = 'dips' THEN l.max_weight ELSE 0 END) as dips,
                            MAX(CASE WHEN l.movement_name = 'squat' THEN l.max_weight ELSE 0 END) as squat,
                            SUM(l.max_weight) as total
                        FROM competition_participants cp
                        INNER JOIN athletes a ON cp.athlete_id = a.athlete_id
                        INNER JOIN competition_groups cg ON cp.group_id = cg.group_id
                        INNER JOIN competitions c ON cg.competition_id = c.competition_id
                        INNER JOIN lifts l ON cp.participant_id = l.participant_id
                        WHERE a.country = $1
                        GROUP BY cp.participant_id, a.athlete_id, a.first_name, a.last_name,
                                 a.slug, a.country, a.gender, c.competition_id, c.name, c.start_date
                    )
                    SELECT
                        ROW_NUMBER() OVER (ORDER BY {} DESC) as rank,
                        athlete_id,
                        first_name,
                        last_name,
                        slug,
                        country,
                        gender,
                        competition_id,
                        competition_name,
                        start_date,
                        total,
                        muscleup,
                        pullup,
                        dips,
                        squat
                    FROM movement_weights
                    ORDER BY {} DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    sort_field, sort_field
                );

                let rows = sqlx::query(&query)
                    .bind(country)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(self.pool)
                    .await?;

                let entries = rows
                    .into_iter()
                    .map(|row| {
                        let rank: i64 = row.get("rank");
                        GlobalRankingEntry {
                            rank: rank + offset,
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
                        }
                    })
                    .collect();

                Ok((entries, total_items))
            }
            (None, None) => {
                let total_items = sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(DISTINCT cp.participant_id)
                    FROM competition_participants cp
                    INNER JOIN lifts l ON cp.participant_id = l.participant_id
                    "#
                )
                .fetch_one(self.pool)
                .await?
                .unwrap_or(0);

                let query = format!(
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
                            c.competition_id,
                            c.name as competition_name,
                            c.start_date,
                            MAX(CASE WHEN l.movement_name = 'muscleup' THEN l.max_weight ELSE 0 END) as muscleup,
                            MAX(CASE WHEN l.movement_name = 'pullup' THEN l.max_weight ELSE 0 END) as pullup,
                            MAX(CASE WHEN l.movement_name = 'dips' THEN l.max_weight ELSE 0 END) as dips,
                            MAX(CASE WHEN l.movement_name = 'squat' THEN l.max_weight ELSE 0 END) as squat,
                            SUM(l.max_weight) as total
                        FROM competition_participants cp
                        INNER JOIN athletes a ON cp.athlete_id = a.athlete_id
                        INNER JOIN competition_groups cg ON cp.group_id = cg.group_id
                        INNER JOIN competitions c ON cg.competition_id = c.competition_id
                        INNER JOIN lifts l ON cp.participant_id = l.participant_id
                        GROUP BY cp.participant_id, a.athlete_id, a.first_name, a.last_name,
                                 a.slug, a.country, a.gender, c.competition_id, c.name, c.start_date
                    )
                    SELECT
                        ROW_NUMBER() OVER (ORDER BY {} DESC) as rank,
                        athlete_id,
                        first_name,
                        last_name,
                        slug,
                        country,
                        gender,
                        competition_id,
                        competition_name,
                        start_date,
                        total,
                        muscleup,
                        pullup,
                        dips,
                        squat
                    FROM movement_weights
                    ORDER BY {} DESC
                    LIMIT $1 OFFSET $2
                    "#,
                    sort_field, sort_field
                );

                let rows = sqlx::query(&query)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(self.pool)
                    .await?;

                let entries = rows
                    .into_iter()
                    .map(|row| {
                        let rank: i64 = row.get("rank");
                        GlobalRankingEntry {
                            rank: rank + offset,
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
                        }
                    })
                    .collect();

                Ok((entries, total_items))
            }
        }
    }
}

fn decimal_to_f64(decimal: Decimal) -> f64 {
    decimal.to_string().parse().unwrap_or(0.0)
}
