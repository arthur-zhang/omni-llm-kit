mod common;
mod http_client;
pub mod model;
mod models;
mod reqwest_client;
pub use reqwest_client::*;
pub mod anthropic;
pub mod openai;
mod tool;

pub use models::*;
pub use model::*;
pub use http_client::*;
pub use tool::*;
#[cfg(test)]
mod tests {
    use crate::model::{LanguageModelRequest, LanguageModelRequestMessage, MessageContent, Role};
    use crate::models::{AnthropicLanguageModelProvider, OpenAiLanguageModelProvider};
    use crate::openai::Model;
    use crate::{anthropic, reqwest_client, AnthropicSettings};
    use futures_util::StreamExt;
    use std::sync::Arc;
    use crate::anthropic::AnthropicModelMode;

    #[tokio::test]
    async fn test_openai_language_model() {
        dotenvy::dotenv().ok();
        let client = Arc::new(reqwest_client::ReqwestClient::new());
        let provider = OpenAiLanguageModelProvider::new(client);
        let model = provider.create_language_model(Model::Custom {
            // name: "kimi-thinking-preview".to_string(),
            // name: "kimi-k2-0711-preview".to_string(),
            name: "aaa".to_string(),
            display_name: Some("kimi-k2-turbo-preview".into()),
            max_tokens: 0,
            max_output_tokens: None,
            max_completion_tokens: None,
        });

        let mut req = LanguageModelRequest::default();

        req.messages = vec![LanguageModelRequestMessage {
            role: Role::User,
            content: vec![MessageContent::Text("请解释 1+1=2。深度思考".into())],
            cache: false,
        }];
        let mut stream = model.stream_completion(req).await.unwrap();
        while let Some(it) = stream.next().await {
            match it {
                Ok(event) => {
                    println!("Event: {:?}", event);
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_anthropic_language_model() {
        dotenvy::dotenv().ok();

        let api_url = std::env::var("ANTHROPIC_API_BASE_URL").unwrap();
        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap();
        let anthropic_settings = AnthropicSettings{
            api_url: api_url,
            api_key: api_key
        };
        global_registry::register_arc!(AnthropicSettings, anthropic_settings);
        let client = Arc::new(reqwest_client::ReqwestClient::new());
        let provider = AnthropicLanguageModelProvider::new(client);
        let model = provider.create_language_model(anthropic::Model::Custom {
            name: "moonshot-v1-8k".to_string(),
            display_name: Some("kimi-k2-turbo-preview".into()),
            tool_override: None,
            max_tokens: 0,
            max_output_tokens: None,
            default_temperature: None,
            extra_beta_headers: vec![],
            cache_configuration: None,
            // mode: Default::default(),
            mode: AnthropicModelMode::Thinking { budget_tokens: None }
        });

        let mut req = LanguageModelRequest::default();

        req.messages = vec![LanguageModelRequestMessage {
            role: Role::User,
            content: vec![MessageContent::Text("who are you".into())],
            cache: false,
        }];
        let mut stream = model.stream_completion(req).await.unwrap();
        while let Some(it) = stream.next().await {
            match it {
                Ok(event) => {
                    println!("Event: {:?}", event);
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                }
            }
        }
    }
}
