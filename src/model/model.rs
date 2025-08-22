use crate::model::errors::LanguageModelCompletionError;
use crate::model::types::{
    LanguageModelCompletionEvent, LanguageModelId, LanguageModelName, LanguageModelProviderId,
    LanguageModelProviderName, LanguageModelToolSchemaFormat,
};
use futures_core::stream::BoxStream;
use crate::model::LanguageModelRequest;

#[async_trait::async_trait]
pub trait LanguageModel: Send + Sync {
    fn id(&self) -> LanguageModelId;
    fn name(&self) -> LanguageModelName;
    fn provider_id(&self) -> LanguageModelProviderId;
    fn provider_name(&self) -> LanguageModelProviderName;
    fn max_token_count(&self) -> u64;
    fn max_output_tokens(&self) -> Option<u64> {
        None
    }
    fn tool_input_format(&self) -> LanguageModelToolSchemaFormat {
        LanguageModelToolSchemaFormat::JsonSchema
    }
    async fn stream_completion(
        &self,
        request: LanguageModelRequest,
    ) -> Result<
        BoxStream<Result<LanguageModelCompletionEvent, LanguageModelCompletionError>>,
        LanguageModelCompletionError,
    >;
    fn supports_tools(&self) -> bool;
    fn supports_burn_mode(&self) -> bool;
}
