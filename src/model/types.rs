use crate::common::{SharedString, is_default};
use crate::model::language_provider::LanguageModelProvider;
use crate::model::model::LanguageModel;
use schemars::_private::serde_json;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd, Serialize, Deserialize)]
pub struct LanguageModelId(pub SharedString);
impl From<String> for LanguageModelId {
    fn from(value: String) -> Self {
        Self(SharedString::from(value))
    }
}
#[derive(Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct LanguageModelName(pub SharedString);
impl From<String> for LanguageModelName {
    fn from(value: String) -> Self {
        Self(SharedString::from(value))
    }
}


#[derive(Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct LanguageModelProviderId(pub SharedString);
impl LanguageModelProviderId {
    pub const fn new(id: &'static str) -> Self {
        Self(SharedString::new_static(id))
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct LanguageModelProviderName(pub SharedString);

impl LanguageModelProviderName {
    pub const fn new(id: &'static str) -> Self {
        Self(SharedString::new_static(id))
    }
}
impl fmt::Display for LanguageModelProviderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}


#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct LanguageModelToolUseId(Arc<str>);
impl fmt::Display for LanguageModelToolUseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl<T> From<T> for LanguageModelToolUseId
where
    T: Into<Arc<str>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct LanguageModelToolUse {
    pub id: LanguageModelToolUseId,
    pub name: Arc<str>,
    pub raw_input: String,
    pub input: serde_json::Value,
    pub is_input_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct LanguageModelToolResult {
    pub tool_use_id: LanguageModelToolUseId,
    pub tool_name: Arc<str>,
    pub is_error: bool,
    pub content: LanguageModelToolResultContent,
    pub output: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash)]
pub enum LanguageModelToolResultContent {
    Text(Arc<str>),
    // Image(LanguageModelImage),
}
impl From<&str> for LanguageModelToolResultContent {
    fn from(value: &str) -> Self {
        Self::Text(Arc::from(value))
    }
}

impl From<String> for LanguageModelToolResultContent {
    fn from(value: String) -> Self {
        Self::Text(Arc::from(value))
    }
}

impl<'de> Deserialize<'de> for LanguageModelToolResultContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let value = serde_json::Value::deserialize(deserializer)?;

        // Models can provide these responses in several styles. Try each in order.

        // 1. Try as plain string
        if let Ok(text) = serde_json::from_value::<String>(value.clone()) {
            return Ok(Self::Text(Arc::from(text)));
        }

        // 2. Try as object
        if let Some(obj) = value.as_object() {
            // get a JSON field case-insensitively
            fn get_field<'a>(
                obj: &'a serde_json::Map<String, serde_json::Value>,
                field: &str,
            ) -> Option<&'a serde_json::Value> {
                obj.iter()
                    .find(|(k, _)| k.to_lowercase() == field.to_lowercase())
                    .map(|(_, v)| v)
            }

            // Accept wrapped text format: { "type": "text", "text": "..." }
            if let (Some(type_value), Some(text_value)) =
                (get_field(&obj, "type"), get_field(&obj, "text"))
            {
                if let Some(type_str) = type_value.as_str() {
                    if type_str.to_lowercase() == "text" {
                        if let Some(text) = text_value.as_str() {
                            return Ok(Self::Text(Arc::from(text)));
                        }
                    }
                }
            }

            // Check for wrapped Text variant: { "text": "..." }
            if let Some((_key, value)) = obj.iter().find(|(k, _)| k.to_lowercase() == "text") {
                if obj.len() == 1 {
                    // Only one field, and it's "text" (case-insensitive)
                    if let Some(text) = value.as_str() {
                        return Ok(Self::Text(Arc::from(text)));
                    }
                }
            }

            // Check for wrapped Image variant: { "image": { "source": "...", "size": ... } }
            if let Some((_key, value)) = obj.iter().find(|(k, _)| k.to_lowercase() == "image") {
                if obj.len() == 1 {
                    // Only one field, and it's "image" (case-insensitive)
                    // Try to parse the nested image object
                    if let Some(image_obj) = value.as_object() {
                        // if let Some(image) = LanguageModelImage::from_json(image_obj) {
                        //     return Ok(Self::Image(image));
                        // }
                        todo!()
                    }
                }
            }

            // Try as direct Image (object with "source" and "size" fields)
            // if let Some(image) = LanguageModelImage::from_json(&obj) {
            //     return Ok(Self::Image(image));
            // }
        }

        // If none of the variants match, return an error with the problematic JSON
        Err(D::Error::custom(format!(
            "data did not match any variant of LanguageModelToolResultContent. Expected either a string, \
             an object with 'type': 'text', a wrapped variant like {{\"Text\": \"...\"}}, or an image object. Got: {}",
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
        )))
    }
}

impl LanguageModelToolResultContent {
    pub fn to_str(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(&text),
            // Self::Image(_) => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Text(text) => text.chars().all(|c| c.is_whitespace()),
            // Self::Image(_) => false,
        }
    }
}
#[derive(Debug, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub struct LanguageModelRequestTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum LanguageModelCompletionEvent {
    StatusUpdate(CompletionRequestStatus),
    Stop(StopReason),
    Text(String),
    Thinking {
        text: String,
        signature: Option<String>,
    },
    RedactedThinking {
        data: String,
    },
    ToolUse(LanguageModelToolUse),
    ToolUseJsonParseError {
        id: LanguageModelToolUseId,
        tool_name: Arc<str>,
        raw_input: Arc<str>,
        json_parse_error: String,
    },
    StartMessage {
        message_id: String,
    },
    UsageUpdate(TokenUsage),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionRequestStatus {
    Queued {
        position: usize,
    },
    Started,
    Failed {
        code: String,
        message: String,
        request_id: Uuid,
        /// Retry duration in seconds.
        retry_after: Option<f64>,
    },
    UsageUpdated {
        amount: usize,
        limit: UsageLimit,
    },
    ToolUseLimitReached,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    #[serde(default, skip_serializing_if = "is_default")]
    pub input_tokens: u64,
    #[serde(default, skip_serializing_if = "is_default")]
    pub output_tokens: u64,
    #[serde(default, skip_serializing_if = "is_default")]
    pub cache_creation_input_tokens: u64,
    #[serde(default, skip_serializing_if = "is_default")]
    pub cache_read_input_tokens: u64,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageLimit {
    Limited(i32),
    Unlimited,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    ToolUse,
    Refusal,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionIntent {
    UserPrompt,
    ToolResults,
    ThreadSummarization,
    ThreadContextSummarization,
    CreateFile,
    EditFile,
    InlineAssist,
    TerminalInlineAssist,
    GenerateGitCommitMessage,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionMode {
    Normal,
    Max,
}

#[derive(Debug, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub enum LanguageModelToolChoice {
    Auto,
    Any,
    None,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum LanguageModelToolSchemaFormat {
    /// A JSON schema, see https://json-schema.org
    JsonSchema,
    /// A subset of an OpenAPI 3.0 schema object supported by Google AI, see https://ai.google.dev/api/caching#Schema
    JsonSchemaSubset,
}

#[derive(Clone)]
pub struct ConfiguredModel {
    pub provider: Arc<dyn LanguageModelProvider>,
    pub model: Arc<dyn LanguageModel>,
}

impl ConfiguredModel {
    pub fn is_same_as(&self, other: &ConfiguredModel) -> bool {
        self.model.id() == other.model.id() && self.provider.id() == other.provider.id()
    }
}
