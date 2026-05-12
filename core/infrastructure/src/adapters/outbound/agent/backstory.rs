use application::ports::{
    BackstoryGenerationRequest, BackstoryGenerationResponse, BackstoryGeneratorError,
    BackstoryGeneratorPort, BackstoryGeneratorResult,
};
use domain::value_objects::AttributeValue;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

const DEFAULT_BASE_URL: &str = "https://kmfj59fk-5000.euw.devtunnels.ms";
const DEFAULT_MAX_NEW_TOKENS: u32 = 180;

/// REST adapter that talks to the local `backstory-model` Flask API.
pub struct BackstoryModelAdapter {
    client: Client,
    base_url: String,
    max_new_tokens: u32,
}

impl BackstoryModelAdapter {
    /// Creates the adapter from environment-driven configuration.
    ///
    /// Supported variables:
    /// - `BACKSTORY_MODEL_BASE_URL`
    /// - `BACKSTORY_MODEL_MAX_NEW_TOKENS`
    pub fn new() -> Result<Self, reqwest::Error> {
        let client = Client::builder().build()?;
        let base_url = normalize_base_url(
            &std::env::var("BACKSTORY_MODEL_BASE_URL")
                .unwrap_or_else(|_| String::from(DEFAULT_BASE_URL)),
        );
        let max_new_tokens = std::env::var("BACKSTORY_MODEL_MAX_NEW_TOKENS")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .map(|value| value.clamp(50, 512))
            .unwrap_or(DEFAULT_MAX_NEW_TOKENS);

        Ok(Self {
            client,
            base_url,
            max_new_tokens,
        })
    }

    fn generate_url(&self) -> String {
        format!("{}/generate", self.base_url)
    }
}

#[async_trait::async_trait]
impl BackstoryGeneratorPort for BackstoryModelAdapter {
    async fn generate_backstory(
        &self,
        request: BackstoryGenerationRequest,
    ) -> BackstoryGeneratorResult<BackstoryGenerationResponse> {
        if request.context_object.title.trim().is_empty()
            || request.context_object.short_desc.trim().is_empty()
        {
            return Err(BackstoryGeneratorError::InvalidQuery {
                details: String::from("NPC title and short_desc must be non-empty"),
            });
        }

        let details = infer_character_details(&request);
        let payload = GenerateBackstoryRequest {
            name: &details.name,
            race: &details.race,
            class_name: &details.class_name,
            max_new_tokens: self.max_new_tokens,
        };

        let response = self
            .client
            .post(self.generate_url())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|error| BackstoryGeneratorError::Unavailable {
                details: error.to_string(),
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| String::from("<failed to read response body>"));

            let details = format_error_details(status, &body);
            let error = match status {
                StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                    BackstoryGeneratorError::InvalidQuery { details }
                }
                StatusCode::UNAUTHORIZED
                | StatusCode::FORBIDDEN
                | StatusCode::TOO_MANY_REQUESTS => BackstoryGeneratorError::Unavailable { details },
                _ => BackstoryGeneratorError::InvalidResponse { details },
            };

            return Err(error);
        }

        let body: GenerateBackstoryResponse =
            response
                .json()
                .await
                .map_err(|error| BackstoryGeneratorError::InvalidResponse {
                    details: error.to_string(),
                })?;

        if !body.success {
            return Err(BackstoryGeneratorError::InvalidResponse {
                details: body
                    .error
                    .unwrap_or_else(|| String::from("backstory model returned success=false")),
            });
        }

        let backstory = body.backstory.unwrap_or_default().trim().to_owned();

        if backstory.is_empty() {
            return Err(BackstoryGeneratorError::InvalidResponse {
                details: String::from("model returned an empty completion"),
            });
        }

        Ok(BackstoryGenerationResponse { backstory })
    }
}

fn normalize_base_url(raw_url: &str) -> String {
    raw_url
        .trim()
        .trim_end_matches('/')
        .strip_suffix("/generate")
        .unwrap_or_else(|| raw_url.trim().trim_end_matches('/'))
        .to_owned()
}

fn format_error_details(status: StatusCode, body: &str) -> String {
    let body = body.trim();
    if body.is_empty() {
        format!("HTTP {status} with empty response body")
    } else {
        format!("HTTP {status}: {body}")
    }
}

#[derive(Debug, Serialize)]
struct GenerateBackstoryRequest<'a> {
    name: &'a str,
    race: &'a str,
    #[serde(rename = "class")]
    class_name: &'a str,
    max_new_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct GenerateBackstoryResponse {
    success: bool,
    backstory: Option<String>,
    error: Option<String>,
}

#[derive(Debug)]
struct CharacterDetails {
    name: String,
    race: String,
    class_name: String,
}

fn infer_character_details(request: &BackstoryGenerationRequest) -> CharacterDetails {
    let object = &request.context_object;
    let context = build_details_context(request);

    let race = attribute_text(&object.attributes, RACE_ATTRIBUTE_KEYS)
        .or_else(|| find_known_value(&context, RACE_KEYWORDS))
        .unwrap_or_else(|| String::from("Unknown"));
    let class_name = attribute_text(&object.attributes, CLASS_ATTRIBUTE_KEYS)
        .or_else(|| find_known_value(&context, CLASS_KEYWORDS))
        .unwrap_or_else(|| String::from("Unknown"));

    CharacterDetails {
        name: object.title.trim().to_owned(),
        race,
        class_name,
    }
}

fn build_details_context(request: &BackstoryGenerationRequest) -> String {
    let object = &request.context_object;
    let mut parts = vec![
        object.title.as_str(),
        object.short_desc.as_str(),
        request.world_summary.as_str(),
        request.player_action.as_str(),
        request.narrator_message.as_str(),
    ];

    if let Some(long_desc) = object.long_desc.as_deref() {
        parts.push(long_desc);
    }

    parts.join("\n")
}

fn attribute_text(
    attributes: &std::collections::HashMap<String, AttributeValue>,
    keys: &[&str],
) -> Option<String> {
    keys.iter().find_map(|wanted_key| {
        attributes.iter().find_map(|(key, value)| {
            if normalize_attribute_key(key) != normalize_attribute_key(wanted_key) {
                return None;
            }

            match value {
                AttributeValue::Text(text) => {
                    let text = text.trim();
                    (!text.is_empty()).then(|| text.to_owned())
                }
                AttributeValue::Number(_) | AttributeValue::Bool(_) => None,
            }
        })
    })
}

fn normalize_attribute_key(value: &str) -> String {
    value
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else {
                None
            }
        })
        .collect()
}

fn find_known_value(context: &str, known_values: &[(&str, &[&str])]) -> Option<String> {
    let normalized_context = normalize_for_search(context);
    if normalized_context.is_empty() {
        return None;
    }

    let haystack = format!(" {normalized_context} ");
    known_values.iter().find_map(|(canonical, aliases)| {
        aliases.iter().find_map(|alias| {
            let normalized_alias = normalize_for_search(alias);
            let needle = format!(" {normalized_alias} ");
            haystack.contains(&needle).then(|| String::from(*canonical))
        })
    })
}

fn normalize_for_search(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    for ch in value.chars() {
        for lower in ch.to_lowercase() {
            if lower.is_alphanumeric() {
                normalized.push(lower);
            } else {
                normalized.push(' ');
            }
        }
    }

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

const RACE_ATTRIBUTE_KEYS: &[&str] = &["race", "ancestry", "species", "heritage", "lineage"];
const CLASS_ATTRIBUTE_KEYS: &[&str] = &[
    "class",
    "character_class",
    "role",
    "profession",
    "occupation",
    "archetype",
    "job",
];

const RACE_KEYWORDS: &[(&str, &[&str])] = &[
    ("Half-Elf", &["half-elf", "half elf"]),
    ("Half-Orc", &["half-orc", "half orc"]),
    ("Dragonborn", &["dragonborn"]),
    ("Tiefling", &["tiefling"]),
    ("Human", &["human"]),
    ("Elf", &["elf"]),
    ("Dwarf", &["dwarf"]),
    ("Gnome", &["gnome"]),
    ("Halfling", &["halfling"]),
    ("Orc", &["orc"]),
    ("Aasimar", &["aasimar"]),
];

const CLASS_KEYWORDS: &[(&str, &[&str])] = &[
    ("Merchant", &["merchant", "trader"]),
    ("Guard", &["guard", "watchman"]),
    ("Soldier", &["soldier"]),
    ("Noble", &["noble", "aristocrat"]),
    ("Priest", &["priest", "acolyte"]),
    ("Blacksmith", &["blacksmith", "smith"]),
    ("Innkeeper", &["innkeeper", "tavern keeper"]),
    ("Alchemist", &["alchemist"]),
    ("Hunter", &["hunter"]),
    ("Mercenary", &["mercenary"]),
    ("Scholar", &["scholar", "sage"]),
    ("Fighter", &["fighter", "warrior"]),
    ("Wizard", &["wizard", "mage"]),
    ("Rogue", &["rogue", "thief"]),
    ("Cleric", &["cleric"]),
    ("Ranger", &["ranger"]),
    ("Paladin", &["paladin"]),
    ("Barbarian", &["barbarian"]),
    ("Bard", &["bard"]),
    ("Druid", &["druid"]),
    ("Monk", &["monk"]),
    ("Sorcerer", &["sorcerer"]),
    ("Warlock", &["warlock"]),
    ("Artificer", &["artificer"]),
];
