use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;
use crate::models::{RisFormulaVersion, RisScoreHistory};
use crate::repository::ris::RisRepository;

pub async fn compute_ris(
    bodyweight: Decimal,
    total: Decimal,
    gender: &str,
    formula: &RisFormulaVersion,
) -> Result<Decimal> {
    let constants = formula.constants_for_gender(gender);

    let bw_minus_v = bodyweight - constants.v;
    let exp_arg = -constants.b * bw_minus_v;

    let exp_term = decimal_exp(exp_arg);
    let denominator_fraction = (constants.k - constants.a) / (Decimal::ONE + constants.q * exp_term);
    let denominator = constants.a + denominator_fraction;

    let ris_score = (total * Decimal::from(100)) / denominator;

    Ok(ris_score.round_dp(2))
}

fn decimal_exp(x: Decimal) -> Decimal {
    let x_f64: f64 = x.to_string().parse().unwrap_or(0.0);
    let result = x_f64.exp();
    Decimal::from_f64_retain(result).unwrap_or(Decimal::ONE)
}

pub async fn get_formula_for_date(pool: &PgPool, competition_date: NaiveDate) -> Result<RisFormulaVersion> {
    let repo = RisRepository::new(pool);
    repo.get_formula_for_date(competition_date).await
}

pub async fn get_current_formula(pool: &PgPool) -> Result<RisFormulaVersion> {
    let repo = RisRepository::new(pool);
    repo.get_current_formula().await
}

pub async fn compute_and_store_ris(
    pool: &PgPool,
    participant_id: Uuid,
    bodyweight: Decimal,
    total: Decimal,
    gender: &str,
) -> Result<Decimal> {
    let repo = RisRepository::new(pool);
    let formula = repo.get_current_formula().await?;

    let ris_score = compute_ris(bodyweight, total, gender, &formula).await?;

    repo.upsert_ris_score(participant_id, formula.formula_id, ris_score, bodyweight, total).await?;
    repo.update_participant_current_ris(participant_id, ris_score).await?;

    Ok(ris_score)
}

pub async fn compute_historical_ris(
    pool: &PgPool,
    participant_id: Uuid,
    bodyweight: Decimal,
    total: Decimal,
    gender: &str,
) -> Result<Vec<RisScoreHistory>> {
    let repo = RisRepository::new(pool);
    let formulas = repo.list_all_formulas().await?;

    let mut results = Vec::new();

    for formula in formulas {
        let ris_score = compute_ris(bodyweight, total, gender, &formula).await?;
        let score_history = repo.upsert_ris_score(
            participant_id,
            formula.formula_id,
            ris_score,
            bodyweight,
            total,
        ).await?;
        results.push(score_history);
    }

    Ok(results)
}

pub async fn recompute_all_ris(pool: &PgPool, formula_id: Option<Uuid>) -> Result<u64> {
    let repo = RisRepository::new(pool);

    let formula = if let Some(fid) = formula_id {
        repo.get_formula_by_id(fid).await?
    } else {
        repo.get_current_formula().await?
    };

    let participants = sqlx::query!(
        r#"
        SELECT
            cp.participant_id,
            cp.bodyweight,
            a.gender,
            COALESCE(SUM(l.max_weight), 0) as "total!: rust_decimal::Decimal"
        FROM competition_participants cp
        INNER JOIN athletes a ON cp.athlete_id = a.athlete_id
        LEFT JOIN lifts l ON l.participant_id = cp.participant_id
        WHERE cp.bodyweight IS NOT NULL
        GROUP BY cp.participant_id, cp.bodyweight, a.gender
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut count = 0u64;

    for participant in participants {
        if let Some(bodyweight) = participant.bodyweight {
            let ris_score = compute_ris(bodyweight, participant.total, &participant.gender, &formula).await?;

            repo.upsert_ris_score(
                participant.participant_id,
                formula.formula_id,
                ris_score,
                bodyweight,
                participant.total,
            ).await?;

            repo.update_participant_current_ris(participant.participant_id, ris_score).await?;
            count += 1;
        }
    }

    Ok(count)
}
