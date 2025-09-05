use crate::model::errors::LanguageModelCompletionError;
use crate::model::types::{
    LanguageModelCompletionEvent, LanguageModelId, LanguageModelName, LanguageModelProviderId,
    LanguageModelProviderName, LanguageModelToolSchemaFormat,
};
use futures_core::stream::BoxStream;
use crate::CompletionMode;
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
    fn max_token_count_in_burn_mode(&self) -> Option<u64> {
        None
    }
}

pub trait LanguageModelExt: LanguageModel {
    fn max_token_count_for_mode(&self, mode: CompletionMode) -> u64 {
        match mode {
            CompletionMode::Normal => self.max_token_count(),
            CompletionMode::Max => self
                .max_token_count_in_burn_mode()
                .unwrap_or_else(|| self.max_token_count()),
        }
    }
}

impl LanguageModelExt for dyn LanguageModel + Send + Sync {}
