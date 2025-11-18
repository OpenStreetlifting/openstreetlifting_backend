use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiResponse {
    pub contest: Contest,
    pub results: ApiResults,
    #[serde(rename = "runningAttemptId")]
    pub running_attempt_id: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Contest {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiResults {
    pub categories: HashMap<String, CategoryInfo>,
    pub results: HashMap<String, HashMap<String, AthleteData>>,
    pub movements: HashMap<String, Movement>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CategoryInfo {
    pub id: i32,
    pub name: String,
    pub genre: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AthleteData {
    #[serde(rename = "athleteInfo")]
    pub athlete_info: AthleteInfo,
    pub results: HashMap<String, MovementResults>,
    pub total: f64,
    #[serde(rename = "RIS")]
    pub ris: f64,
    pub rank: AthleteRank,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum AthleteRank {
    Position(u32),
    Disqualified(String),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AthleteInfo {
    pub id: i32,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    pub pesee: Option<f64>,
    #[serde(rename = "isOut")]
    pub is_out: bool,
    #[serde(rename = "reasonOut")]
    pub reason_out: Option<String>,
    #[serde(rename = "reglageDips")]
    pub reglage_dips: Option<String>,
    #[serde(rename = "reglageSquat")]
    pub reglage_squat: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MovementResults {
    pub results: HashMap<String, Option<Attempt>>,
    pub max: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Attempt {
    pub id: i32,
    #[serde(rename = "noEssai")]
    pub no_essai: i32,
    pub charge: f64,
    #[serde(rename = "decisionRep")]
    pub decision_rep: DecisionRep,
    #[serde(rename = "justificationNoRep")]
    pub justification_no_rep: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum DecisionRep {
    Number(i32),
    String(String),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Movement {
    pub id: i32,
    pub name: String,
    pub order: i32,
}
