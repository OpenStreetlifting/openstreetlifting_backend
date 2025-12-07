use crate::canonical::models::CanonicalFormat;

pub struct PromptBuilder;

impl PromptBuilder {
    pub fn system_prompt() -> String {
        format!(
            r#"You are a competition data extraction assistant. Extract structured data and output ONLY valid JSON.

Schema:
{}

Rules:
1. Movement normalization:
   - "Traction/Tractions/Pull up" → "Pull-up"
   - "Muscle up/Muscleup" → "Muscle-up"
   - "Dip" → "Dips"

2. Data handling:
   - Infer missing data when obvious
   - Use null for missing data
   - Parse categories into gender and weight classes
   - Extract all attempts with weights and success status

3. Output:
   - Return ONLY valid JSON, no explanations
   - Use ISO 8601 dates (YYYY-MM-DD)
   - Ensure all required fields present"#,
            Self::schema_example()
        )
    }

    pub fn user_prompt_html(html: &str) -> String {
        format!("Extract competition data from this HTML:\n\n{}", html)
    }

    pub fn user_prompt_csv(csv: &str) -> String {
        format!("Extract competition data from this CSV:\n\n{}", csv)
    }

    pub fn user_prompt_image() -> String {
        "Extract competition data from this image. The image shows competition results in a table format.".to_string()
    }

    fn schema_example() -> String {
        let example = CanonicalFormat {
            format_version: "1.0.0".to_string(),
            source: crate::canonical::models::SourceMetadata {
                r#type: crate::canonical::models::SourceType::Html,
                url: Some("https://example.com".to_string()),
                extracted_at: chrono::Utc::now(),
                extractor: "ollama-qwen-2.5-7b".to_string(),
                original_filename: None,
            },
            competition: crate::canonical::models::CompetitionData {
                name: "Competition Name".to_string(),
                slug: "competition-slug".to_string(),
                federation: crate::canonical::models::FederationData {
                    name: "Federation Name".to_string(),
                    slug: Some("federation-slug".to_string()),
                    abbreviation: Some("FED".to_string()),
                    country: Some("Country".to_string()),
                },
                start_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
                end_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
                venue: Some("Venue Name".to_string()),
                city: Some("City".to_string()),
                country: "Country".to_string(),
                number_of_judges: Some(3),
                status: Some("completed".to_string()),
            },
            movements: vec![crate::canonical::models::MovementData {
                name: "Pull-up".to_string(),
                order: 1,
                is_required: Some(true),
            }],
            categories: vec![crate::canonical::models::CategoryData {
                name: "Men -73kg".to_string(),
                gender: "M".to_string(),
                weight_class_min: None,
                weight_class_max: Some(rust_decimal::Decimal::new(73, 0)),
                athletes: vec![],
            }],
            liftcontrol_metadata: None,
            pdf_metadata: None,
        };

        serde_json::to_string_pretty(&example).unwrap_or_default()
    }
}
