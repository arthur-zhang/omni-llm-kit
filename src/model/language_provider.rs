use crate::model::model::LanguageModel;
use crate::model::types::{LanguageModelProviderId, LanguageModelProviderName};
use std::sync::Arc;

#[async_trait::async_trait]
pub trait LanguageModelProvider: 'static {
    fn id(&self) -> LanguageModelProviderId;
    fn name(&self) -> LanguageModelProviderName;
    fn default_model(&self) -> Option<Arc<dyn LanguageModel>>;
    fn default_fast_model(&self) -> Option<Arc<dyn LanguageModel>> {
        todo!()
    }
    fn provided_models(&self) -> Vec<Arc<dyn LanguageModel>>;
    // async fn authenticate(&self) -> anyhow::Result<()>;
}
