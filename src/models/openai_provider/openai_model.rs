use anyhow::anyhow;
use futures_core::Stream;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use schemars::JsonSchema;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
// use futures_core::{future::BoxFuture, stream::{BoxStream};

use futures_util::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use log::info;
use crate::http_client::HttpClient;
use crate::model::{LanguageModel, LanguageModelCompletionError, LanguageModelCompletionEvent, LanguageModelId, LanguageModelName, LanguageModelProviderId, LanguageModelProviderName, LanguageModelRequest, LanguageModelToolChoice, LanguageModelToolResultContent, MessageContent, Role};
use crate::models::openai_provider::event_mapper::OpenAiEventMapper;
use crate::openai::{self, ImageUrl, ResponseStreamEvent};

pub const OPEN_AI_PROVIDER_ID: LanguageModelProviderId = LanguageModelProviderId::new("openai");
pub const OPEN_AI_PROVIDER_NAME: LanguageModelProviderName =
    LanguageModelProviderName::new("OpenAI");

pub struct OpenAiLanguageModel {
    pub(crate) id: LanguageModelId,
    pub(crate) model: openai::Model,
    // pub(crate) state: State,
    pub(crate) http_client: Arc<dyn HttpClient>,
}

impl OpenAiLanguageModel {
    async fn stream_completion(
        &self,
        request: openai::Request,
    ) -> anyhow::Result<BoxStream<'static, anyhow::Result<ResponseStreamEvent>>> {
        let http_client = self.http_client.clone();
        //
        // let settings = App::global::<OpenAiSettings>();
        // let api_url = settings.api_url.as_str();
        // println!("OpenAI API URL: {}", api_url);
        // let api_key = self
        //     .state
        //     .api_key();
        // let api_key = api_key
        //     .as_deref()
        //     .ok_or(anyhow!("api key not found"))?;
        let api_key = std::env::var("OPENAI_API_KEY")?;
        let base_url = std::env::var("OPENAI_BASE_URL")?;

        let response =
            openai::stream_completion(http_client.as_ref(), &base_url, &api_key, request).await?;
        Ok(response.boxed())
    }
}
#[async_trait::async_trait]
impl LanguageModel for OpenAiLanguageModel {
    fn id(&self) -> LanguageModelId {
        self.id.clone()
    }

    fn name(&self) -> LanguageModelName {
        LanguageModelName::from(self.model.display_name().to_string())
    }

    fn provider_id(&self) -> LanguageModelProviderId {
        OPEN_AI_PROVIDER_ID
    }

    fn provider_name(&self) -> LanguageModelProviderName {
        OPEN_AI_PROVIDER_NAME
    }

    fn max_token_count(&self) -> u64 {
        self.model.max_token_count()
    }
    fn max_output_tokens(&self) -> Option<u64> {
        self.model.max_output_tokens()
    }
    fn supports_tools(&self) -> bool {
        return true;
    }
    fn supports_burn_mode(&self) -> bool {
        return false;
    }
    async fn stream_completion(
        &self,
        request: LanguageModelRequest,
    ) -> Result<
        BoxStream<Result<LanguageModelCompletionEvent, LanguageModelCompletionError>>,
        LanguageModelCompletionError,
    > {
        let request = into_open_ai(
            request,
            self.model.id(),
            self.model.supports_parallel_tool_calls(),
            self.max_output_tokens(),
        );
        let completion = self.stream_completion(request).await?.boxed();
        let mapper = OpenAiEventMapper::new();
        Ok(mapper.map_stream(completion).boxed())
    }
}
fn add_message_content_part(
    new_part: openai::MessagePart,
    role: Role,
    messages: &mut Vec<openai::RequestMessage>,
) {
    match (role, messages.last_mut()) {
        (Role::User, Some(openai::RequestMessage::User { content }))
        | (
            Role::Assistant,
            Some(openai::RequestMessage::Assistant {
                     content: Some(content),
                     ..
                 }),
        )
        | (Role::System, Some(openai::RequestMessage::System { content, .. })) => {
            content.push_part(new_part);
        }
        _ => {
            messages.push(match role {
                Role::User => openai::RequestMessage::User {
                    content: openai::MessageContent::from(vec![new_part]),
                },
                Role::Assistant => openai::RequestMessage::Assistant {
                    content: Some(openai::MessageContent::from(vec![new_part])),
                    tool_calls: Vec::new(),
                },
                Role::System => openai::RequestMessage::System {
                    content: openai::MessageContent::from(vec![new_part]),
                },
            });
        }
    }
}
pub fn into_open_ai(
    request: LanguageModelRequest,
    model_id: &str,
    supports_parallel_tool_calls: bool,
    max_output_tokens: Option<u64>,
) -> openai::Request {
    let stream = !model_id.starts_with("o1-");

    let mut messages = Vec::new();
    for message in request.messages {
        for content in message.content {
            match content {
                MessageContent::Text(text) | MessageContent::Thinking { text, .. } => {
                    add_message_content_part(
                        openai::MessagePart::Text { text: text },
                        message.role,
                        &mut messages,
                    )
                }
                MessageContent::RedactedThinking(_) => {}
                MessageContent::Image(image) => {
                    add_message_content_part(
                        openai::MessagePart::Image {
                            image_url: ImageUrl {
                                url: image.to_base64_url(),
                                detail: None,
                            },
                        },
                        message.role,
                        &mut messages,
                    );
                }
                MessageContent::ToolUse(tool_use) => {
                    let tool_call = openai::ToolCall {
                        id: tool_use.id.to_string(),
                        content: openai::ToolCallContent::Function {
                            function: openai::FunctionContent {
                                name: tool_use.name.to_string(),
                                arguments: serde_json::to_string(&tool_use.input)
                                    .unwrap_or_default(),
                            },
                        },
                    };

                    if let Some(openai::RequestMessage::Assistant { tool_calls, .. }) =
                        messages.last_mut()
                    {
                        tool_calls.push(tool_call);
                    } else {
                        messages.push(openai::RequestMessage::Assistant {
                            content: None,
                            tool_calls: vec![tool_call],
                        });
                    }
                }
                MessageContent::ToolResult(tool_result) => {
                    let content = match &tool_result.content {
                        LanguageModelToolResultContent::Text(text) => {
                            vec![openai::MessagePart::Text {
                                text: text.to_string(),
                            }]
                        } // LanguageModelToolResultContent::Image(image) => {
                        //     vec![openai::MessagePart::Image {
                        //         image_url: ImageUrl {
                        //             url: image.to_base64_url(),
                        //             detail: None,
                        //         },
                        //     }]
                        // }
                    };

                    messages.push(openai::RequestMessage::Tool {
                        content: content.into(),
                        tool_call_id: tool_result.tool_use_id.to_string(),
                    });
                }
            }
        }
    }

    openai::Request {
        model: model_id.into(),
        messages,
        stream,
        stop: request.stop,
        temperature: request.temperature.unwrap_or(1.0),
        max_completion_tokens: max_output_tokens,
        parallel_tool_calls: if supports_parallel_tool_calls && !request.tools.is_empty() {
            // Disable parallel tool calls, as the Agent currently expects a maximum of one per turn.
            Some(false)
        } else {
            None
        },
        tools: request
            .tools
            .into_iter()
            .map(|tool| openai::ToolDefinition::Function {
                function: openai::FunctionDefinition {
                    name: tool.name,
                    description: Some(tool.description),
                    parameters: Some(tool.input_schema),
                },
            })
            .collect(),
        tool_choice: request.tool_choice.map(|choice| match choice {
            LanguageModelToolChoice::Auto => openai::ToolChoice::Auto,
            LanguageModelToolChoice::Any => openai::ToolChoice::Required,
            LanguageModelToolChoice::None => openai::ToolChoice::None,
        }),
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AvailableModel {
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub max_output_tokens: Option<u64>,
    pub max_completion_tokens: Option<u64>,
}
