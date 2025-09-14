use crate::anthropic::{
    AnthropicError, AnthropicModelMode, ContentDelta, Event, ResponseContent, ToolResultContent,
    ToolResultPart, Usage,
};
use anyhow::{Context as _,  anyhow};
use futures::Stream;
use futures::{FutureExt, StreamExt, future::BoxFuture, stream::BoxStream};
use std::collections::{BTreeMap, HashMap};

use crate::http_client::HttpClient;
use crate::model::{
    self, LanguageModel, LanguageModelCompletionError, LanguageModelId, LanguageModelName,
    LanguageModelProvider, LanguageModelProviderId, LanguageModelProviderName,
    LanguageModelRequest, LanguageModelToolChoice, LanguageModelToolResultContent, MessageContent,
    Role,
};
use crate::model::{LanguageModelCompletionEvent, LanguageModelToolUse, StopReason};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use strum::IntoEnumIterator;
// use util::ResultExt;
use crate::anthropic;

const PROVIDER_ID: LanguageModelProviderId = model::ANTHROPIC_PROVIDER_ID;
const PROVIDER_NAME: LanguageModelProviderName = model::ANTHROPIC_PROVIDER_NAME;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct AnthropicSettings {
    pub api_url: String,
    // pub available_models: Vec<AvailableModel>,
    pub api_key: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AvailableModel {
    /// The model's name in the Anthropic API. e.g. claude-3-5-sonnet-latest, claude-3-opus-20240229, etc
    pub name: String,
    /// The model's name in Zed's UI, such as in the model selector dropdown menu in the assistant panel.
    pub display_name: Option<String>,
    /// The model's context window size.
    pub max_tokens: u64,
    /// A model `name` to substitute when calling tools, in case the primary model doesn't support tool calling.
    pub tool_override: Option<String>,
    /// Configuration of Anthropic's caching API.
    // pub cache_configuration: Option<LanguageModelCacheConfiguration>,
    pub max_output_tokens: Option<u64>,
    pub default_temperature: Option<f32>,
    #[serde(default)]
    pub extra_beta_headers: Vec<String>,
    /// The model's mode (e.g. thinking)
    pub mode: Option<ModelMode>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ModelMode {
    #[default]
    Default,
    Thinking {
        /// The maximum number of tokens to use for reasoning. Must be lower than the model's `max_output_tokens`.
        budget_tokens: Option<u32>,
    },
}

impl From<ModelMode> for AnthropicModelMode {
    fn from(value: ModelMode) -> Self {
        match value {
            ModelMode::Default => AnthropicModelMode::Default,
            ModelMode::Thinking { budget_tokens } => AnthropicModelMode::Thinking { budget_tokens },
        }
    }
}

impl From<AnthropicModelMode> for ModelMode {
    fn from(value: AnthropicModelMode) -> Self {
        match value {
            AnthropicModelMode::Default => ModelMode::Default,
            AnthropicModelMode::Thinking { budget_tokens } => ModelMode::Thinking { budget_tokens },
        }
    }
}

pub struct AnthropicLanguageModelProvider {
    http_client: Arc<dyn HttpClient>,
    // state: gpui::Entity<State>,
}

const ANTHROPIC_API_KEY_VAR: &str = "ANTHROPIC_API_KEY";


impl AnthropicLanguageModelProvider {
    pub fn new(http_client: Arc<dyn HttpClient>) -> Self {
        Self { http_client }
    }

    pub fn create_language_model(&self, model: anthropic::Model) -> Arc<dyn LanguageModel> {
        Arc::new(AnthropicModel {
            id: LanguageModelId::from(model.id().to_string()),
            model,
            http_client: self.http_client.clone(),
        })
    }
}

impl LanguageModelProvider for AnthropicLanguageModelProvider {
    fn id(&self) -> LanguageModelProviderId {
        PROVIDER_ID
    }

    fn name(&self) -> LanguageModelProviderName {
        PROVIDER_NAME
    }

    fn default_model(&self) -> Option<Arc<dyn LanguageModel>> {
        Some(self.create_language_model(anthropic::Model::default()))
    }

    fn default_fast_model(&self) -> Option<Arc<dyn LanguageModel>> {
        Some(self.create_language_model(anthropic::Model::default_fast()))
    }

    // fn recommended_models(&self) -> Vec<Arc<dyn LanguageModel>> {
    //     [
    //         anthropic::Model::ClaudeSonnet4,
    //         anthropic::Model::ClaudeSonnet4Thinking,
    //     ]
    //     .into_iter()
    //     .map(|model| self.create_language_model(model))
    //     .collect()
    // }

    fn provided_models(&self) -> Vec<Arc<dyn LanguageModel>> {
        // let mut models = BTreeMap::default();
        //
        // // Add base models from anthropic::Model::iter()
        // for model in anthropic::Model::iter() {
        //     if !matches!(model, anthropic::Model::Custom { .. }) {
        //         models.insert(model.id().to_string(), model);
        //     }
        // }
        //
        // // Override with available models from settings
        // for model in AllLanguageModelSettings::get_global(cx)
        //     .anthropic
        //     .available_models
        //     .iter()
        // {
        //     models.insert(
        //         model.name.clone(),
        //         anthropic::Model::Custom {
        //             name: model.name.clone(),
        //             display_name: model.display_name.clone(),
        //             max_tokens: model.max_tokens,
        //             tool_override: model.tool_override.clone(),
        //             cache_configuration: model.cache_configuration.as_ref().map(|config| {
        //                 anthropic::AnthropicModelCacheConfiguration {
        //                     max_cache_anchors: config.max_cache_anchors,
        //                     should_speculate: config.should_speculate,
        //                     min_total_token: config.min_total_token,
        //                 }
        //             }),
        //             max_output_tokens: model.max_output_tokens,
        //             default_temperature: model.default_temperature,
        //             extra_beta_headers: model.extra_beta_headers.clone(),
        //             mode: model.mode.clone().unwrap_or_default().into(),
        //         },
        //     );
        // }
        //
        // models
        //     .into_values()
        //     .map(|model| self.create_language_model(model))
        //     .collect()
        todo!()
    }

    // fn is_authenticated(&self, cx: &App) -> bool {
    //     self.state.read(cx).is_authenticated()
    // }

    // fn authenticate(&self, cx: &mut App) -> Task<Result<(), AuthenticateError>> {
    //     self.state.update(cx, |state, cx| state.authenticate(cx))
    // }
}

pub struct AnthropicModel {
    id: LanguageModelId,
    model: anthropic::Model,
    http_client: Arc<dyn HttpClient>,
    // request_limiter: RateLimiter,
}

pub fn count_anthropic_tokens(
    request: LanguageModelRequest,
) -> BoxFuture<'static, anyhow::Result<u64>> {
    async move {
        let messages = request.messages;
        let tokens_from_images = 0;
        let mut string_messages = Vec::with_capacity(messages.len());

        for message in messages {
            use crate::model::MessageContent;

            let mut string_contents = String::new();

            for content in message.content {
                match content {
                    MessageContent::Text(text) => {
                        string_contents.push_str(&text);
                    }
                    MessageContent::Thinking { .. } => {
                        // Thinking blocks are not included in the input token count.
                    }
                    MessageContent::RedactedThinking(_) => {
                        // Thinking blocks are not included in the input token count.
                    }
                    MessageContent::Image(image) => {
                        // tokens_from_images += image.estimate_tokens();
                        // todo
                    }
                    MessageContent::ToolUse(_tool_use) => {
                        // TODO: Estimate token usage from tool uses.
                    }
                    MessageContent::ToolResult(tool_result) => match &tool_result.content {
                        LanguageModelToolResultContent::Text(text) => {
                            string_contents.push_str(text);
                        } // LanguageModelToolResultContent::Image(image) => {
                        //     tokens_from_images += image.estimate_tokens();
                        // }
                    },
                }
            }

            if !string_contents.is_empty() {
                string_messages.push(tiktoken_rs::ChatCompletionRequestMessage {
                    role: match message.role {
                        Role::User => "user".into(),
                        Role::Assistant => "assistant".into(),
                        Role::System => "system".into(),
                    },
                    content: Some(string_contents),
                    name: None,
                    function_call: None,
                });
            }
        }

        // Tiktoken doesn't yet support these models, so we manually use the
        // same tokenizer as GPT-4.
        tiktoken_rs::num_tokens_from_messages("gpt-4", &string_messages)
            .map(|tokens| (tokens + tokens_from_images) as u64)
    }
        .boxed()
}

impl AnthropicModel {
    async fn stream_completion(
        &self,
        request: anthropic::Request,
    ) ->
        Result<
            BoxStream<'static, Result<anthropic::Event, AnthropicError>>,
            LanguageModelCompletionError,
        >
    {
        let http_client = self.http_client.clone();

        let anthropic_settings =
            global_registry::get!(AnthropicSettings).expect("AnthropicSettings not found");
        let api_key = anthropic_settings.api_key.clone();
        let api_url = anthropic_settings.api_url.clone();

         anthropic::stream_completion(http_client.as_ref(), &api_url, &api_key, request).await
            .map_err(Into::<LanguageModelCompletionError>::into)
    }
}
#[async_trait::async_trait]
impl LanguageModel for AnthropicModel {
    fn id(&self) -> LanguageModelId {
        self.id.clone()
    }

    fn name(&self) -> LanguageModelName {
        LanguageModelName::from(self.model.display_name().to_string())
    }

    fn provider_id(&self) -> LanguageModelProviderId {
        PROVIDER_ID
    }

    fn provider_name(&self) -> LanguageModelProviderName {
        PROVIDER_NAME
    }

    fn max_token_count(&self) -> u64 {
        self.model.max_token_count()
    }

    fn max_output_tokens(&self) -> Option<u64> {
        Some(self.model.max_output_tokens())
    }

    async fn stream_completion(
        &self,
        request: LanguageModelRequest,
    ) -> Result<
        BoxStream<Result<LanguageModelCompletionEvent, LanguageModelCompletionError>>,
        LanguageModelCompletionError,
    > {
        let request = into_anthropic(
            request,
            self.model.request_id().into(),
            self.model.default_temperature(),
            self.model.max_output_tokens(),
            self.model.mode(),
        );
        let response = self.stream_completion(request).await?;
        let stream = AnthropicEventMapper::new().map_stream(response);
        Ok(stream.boxed())
    }


    fn supports_tools(&self) -> bool {
        true
    }

    fn supports_burn_mode(&self) -> bool {
        true
    }
}

pub fn into_anthropic(
    request: LanguageModelRequest,
    model: String,
    default_temperature: f32,
    max_output_tokens: u64,
    mode: AnthropicModelMode,
) -> anthropic::Request {
    let mut new_messages: Vec<anthropic::Message> = Vec::new();
    let mut system_message = String::new();

    for message in request.messages {
        if message.contents_empty() {
            continue;
        }

        match message.role {
            Role::User | Role::Assistant => {
                let mut anthropic_message_content: Vec<anthropic::RequestContent> = message
                    .content
                    .into_iter()
                    .filter_map(|content| match content {
                        MessageContent::Text(text) => {
                            let text = if text.chars().last().map_or(false, |c| c.is_whitespace()) {
                                text.trim_end().to_string()
                            } else {
                                text
                            };
                            if !text.is_empty() {
                                Some(anthropic::RequestContent::Text {
                                    text,
                                    cache_control: None,
                                })
                            } else {
                                None
                            }
                        }
                        MessageContent::Thinking {
                            text: thinking,
                            signature,
                        } => {
                            if !thinking.is_empty() {
                                Some(anthropic::RequestContent::Thinking {
                                    thinking,
                                    signature: signature.unwrap_or_default(),
                                    cache_control: None,
                                })
                            } else {
                                None
                            }
                        }
                        MessageContent::RedactedThinking(data) => {
                            if !data.is_empty() {
                                Some(anthropic::RequestContent::RedactedThinking { data })
                            } else {
                                None
                            }
                        }
                        MessageContent::Image(image) => Some(anthropic::RequestContent::Image {
                            source: anthropic::ImageSource {
                                source_type: "base64".to_string(),
                                media_type: "image/png".to_string(),
                                data: image.source.to_string(),
                            },
                            cache_control: None,
                        }),
                        MessageContent::ToolUse(tool_use) => {
                            Some(anthropic::RequestContent::ToolUse {
                                id: tool_use.id.to_string(),
                                name: tool_use.name.to_string(),
                                input: tool_use.input,
                                cache_control: None,
                            })
                        }
                        MessageContent::ToolResult(tool_result) => {
                            Some(anthropic::RequestContent::ToolResult {
                                tool_use_id: tool_result.tool_use_id.to_string(),
                                is_error: tool_result.is_error,
                                content: match tool_result.content {
                                    LanguageModelToolResultContent::Text(text) => {
                                        ToolResultContent::Plain(text.to_string())
                                    } // LanguageModelToolResultContent::Image(image) => {
                                    //     ToolResultContent::Multipart(vec![ToolResultPart::Image {
                                    //         source: anthropic::ImageSource {
                                    //             source_type: "base64".to_string(),
                                    //             media_type: "image/png".to_string(),
                                    //             data: image.source.to_string(),
                                    //         },
                                    //     }])
                                    // }
                                },
                                cache_control: None,
                            })
                        }
                    })
                    .collect();
                let anthropic_role = match message.role {
                    Role::User => anthropic::Role::User,
                    Role::Assistant => anthropic::Role::Assistant,
                    Role::System => unreachable!("System role should never occur here"),
                };
                if let Some(last_message) = new_messages.last_mut() {
                    if last_message.role == anthropic_role {
                        last_message.content.extend(anthropic_message_content);
                        continue;
                    }
                }

                // Mark the last segment of the message as cached
                if message.cache {
                    let cache_control_value = Some(anthropic::CacheControl {
                        cache_type: anthropic::CacheControlType::Ephemeral,
                    });
                    for message_content in anthropic_message_content.iter_mut().rev() {
                        match message_content {
                            anthropic::RequestContent::RedactedThinking { .. } => {
                                // Caching is not possible, fallback to next message
                            }
                            anthropic::RequestContent::Text { cache_control, .. }
                            | anthropic::RequestContent::Thinking { cache_control, .. }
                            | anthropic::RequestContent::Image { cache_control, .. }
                            | anthropic::RequestContent::ToolUse { cache_control, .. }
                            | anthropic::RequestContent::ToolResult { cache_control, .. } => {
                                *cache_control = cache_control_value;
                                break;
                            }
                        }
                    }
                }

                new_messages.push(anthropic::Message {
                    role: anthropic_role,
                    content: anthropic_message_content,
                });
            }
            Role::System => {
                if !system_message.is_empty() {
                    system_message.push_str("\n\n");
                }
                system_message.push_str(&message.string_contents());
            }
        }
    }

    anthropic::Request {
        model,
        messages: new_messages,
        max_tokens: max_output_tokens,
        system: if system_message.is_empty() {
            None
        } else {
            Some(anthropic::StringOrContents::String(system_message))
        },
        thinking: if request.thinking_allowed
            && let AnthropicModelMode::Thinking { budget_tokens } = mode
        {
            Some(anthropic::Thinking::Enabled { budget_tokens })
        } else {
            None
        },
        tools: request
            .tools
            .into_iter()
            .map(|tool| anthropic::Tool {
                name: tool.name,
                description: tool.description,
                input_schema: tool.input_schema,
            })
            .collect(),
        tool_choice: request.tool_choice.map(|choice| match choice {
            LanguageModelToolChoice::Auto => anthropic::ToolChoice::Auto,
            LanguageModelToolChoice::Any => anthropic::ToolChoice::Any,
            LanguageModelToolChoice::None => anthropic::ToolChoice::None,
        }),
        metadata: None,
        stop_sequences: Vec::new(),
        temperature: request.temperature.or(Some(default_temperature)),
        top_k: None,
        top_p: None,
    }
}

pub struct AnthropicEventMapper {
    tool_uses_by_index: HashMap<usize, RawToolUse>,
    usage: Usage,
    stop_reason: StopReason,
}

impl AnthropicEventMapper {
    pub fn new() -> Self {
        Self {
            tool_uses_by_index: HashMap::default(),
            usage: Usage::default(),
            stop_reason: StopReason::EndTurn,
        }
    }

    pub fn map_stream(
        mut self,
        events: Pin<Box<dyn Send + Stream<Item=Result<Event, AnthropicError>>>>,
    ) -> impl Stream<Item=Result<LanguageModelCompletionEvent, LanguageModelCompletionError>>
    {
        events.flat_map(move |event| {
            futures::stream::iter(match event {
                Ok(event) => self.map_event(event),
                Err(error) => vec![Err(error.into())],
            })
        })
    }

    pub fn map_event(
        &mut self,
        event: Event,
    ) -> Vec<Result<LanguageModelCompletionEvent, LanguageModelCompletionError>> {
        match event {
            Event::ContentBlockStart {
                index,
                content_block,
            } => match content_block {
                ResponseContent::Text { text } => {
                    vec![Ok(LanguageModelCompletionEvent::Text(text))]
                }
                ResponseContent::Thinking { thinking } => {
                    vec![Ok(LanguageModelCompletionEvent::Thinking {
                        text: thinking,
                        signature: None,
                    })]
                }
                ResponseContent::RedactedThinking { data } => {
                    vec![Ok(LanguageModelCompletionEvent::RedactedThinking { data })]
                }
                ResponseContent::ToolUse { id, name, .. } => {
                    self.tool_uses_by_index.insert(
                        index,
                        RawToolUse {
                            id,
                            name,
                            input_json: String::new(),
                        },
                    );
                    Vec::new()
                }
            },
            Event::ContentBlockDelta { index, delta } => match delta {
                ContentDelta::TextDelta { text } => {
                    vec![Ok(LanguageModelCompletionEvent::Text(text))]
                }
                ContentDelta::ThinkingDelta { thinking } => {
                    vec![Ok(LanguageModelCompletionEvent::Thinking {
                        text: thinking,
                        signature: None,
                    })]
                }
                ContentDelta::SignatureDelta { signature } => {
                    vec![Ok(LanguageModelCompletionEvent::Thinking {
                        text: "".to_string(),
                        signature: Some(signature),
                    })]
                }
                ContentDelta::InputJsonDelta { partial_json } => {
                    if let Some(tool_use) = self.tool_uses_by_index.get_mut(&index) {
                        tool_use.input_json.push_str(&partial_json);

                        // Try to convert invalid (incomplete) JSON into
                        // valid JSON that serde can accept, e.g. by closing
                        // unclosed delimiters. This way, we can update the
                        // UI with whatever has been streamed back so far.
                        if let Ok(input) = serde_json::Value::from_str(
                            &partial_json_fixer::fix_json(&tool_use.input_json),
                        ) {
                            return vec![Ok(LanguageModelCompletionEvent::ToolUse(
                                LanguageModelToolUse {
                                    id: tool_use.id.clone().into(),
                                    name: tool_use.name.clone().into(),
                                    is_input_complete: false,
                                    raw_input: tool_use.input_json.clone(),
                                    input,
                                },
                            ))];
                        }
                    }
                    return vec![];
                }
            },
            Event::ContentBlockStop { index } => {
                if let Some(tool_use) = self.tool_uses_by_index.remove(&index) {
                    let input_json = tool_use.input_json.trim();
                    let input_value = if input_json.is_empty() {
                        Ok(serde_json::Value::Object(serde_json::Map::default()))
                    } else {
                        serde_json::Value::from_str(input_json)
                    };
                    let event_result = match input_value {
                        Ok(input) => Ok(LanguageModelCompletionEvent::ToolUse(
                            LanguageModelToolUse {
                                id: tool_use.id.into(),
                                name: tool_use.name.into(),
                                is_input_complete: true,
                                input,
                                raw_input: tool_use.input_json.clone(),
                            },
                        )),
                        Err(json_parse_err) => {
                            Ok(LanguageModelCompletionEvent::ToolUseJsonParseError {
                                id: tool_use.id.into(),
                                tool_name: tool_use.name.into(),
                                raw_input: input_json.into(),
                                json_parse_error: json_parse_err.to_string(),
                            })
                        }
                    };

                    vec![event_result]
                } else {
                    Vec::new()
                }
            }
            Event::MessageStart { message } => {
                update_usage(&mut self.usage, &message.usage);
                vec![
                    Ok(LanguageModelCompletionEvent::UsageUpdate(convert_usage(
                        &self.usage,
                    ))),
                    Ok(LanguageModelCompletionEvent::StartMessage {
                        message_id: message.id,
                    }),
                ]
            }
            Event::MessageDelta { delta, usage } => {
                update_usage(&mut self.usage, &usage);
                if let Some(stop_reason) = delta.stop_reason.as_deref() {
                    self.stop_reason = match stop_reason {
                        "end_turn" => StopReason::EndTurn,
                        "max_tokens" => StopReason::MaxTokens,
                        "tool_use" => StopReason::ToolUse,
                        "refusal" => StopReason::Refusal,
                        _ => {
                            log::error!("Unexpected anthropic stop_reason: {stop_reason}");
                            StopReason::EndTurn
                        }
                    };
                }
                vec![Ok(LanguageModelCompletionEvent::UsageUpdate(
                    convert_usage(&self.usage),
                ))]
            }
            Event::MessageStop => {
                vec![Ok(LanguageModelCompletionEvent::Stop(self.stop_reason))]
            }
            Event::Error { error } => {
                vec![Err(error.into())]
            }
            _ => Vec::new(),
        }
    }
}

struct RawToolUse {
    id: String,
    name: String,
    input_json: String,
}

/// Updates usage data by preferring counts from `new`.
fn update_usage(usage: &mut Usage, new: &Usage) {
    if let Some(input_tokens) = new.input_tokens {
        usage.input_tokens = Some(input_tokens);
    }
    if let Some(output_tokens) = new.output_tokens {
        usage.output_tokens = Some(output_tokens);
    }
    if let Some(cache_creation_input_tokens) = new.cache_creation_input_tokens {
        usage.cache_creation_input_tokens = Some(cache_creation_input_tokens);
    }
    if let Some(cache_read_input_tokens) = new.cache_read_input_tokens {
        usage.cache_read_input_tokens = Some(cache_read_input_tokens);
    }
}

fn convert_usage(usage: &Usage) -> model::TokenUsage {
    model::TokenUsage {
        input_tokens: usage.input_tokens.unwrap_or(0),
        output_tokens: usage.output_tokens.unwrap_or(0),
        cache_creation_input_tokens: usage.cache_creation_input_tokens.unwrap_or(0),
        cache_read_input_tokens: usage.cache_read_input_tokens.unwrap_or(0),
    }
}
