use crate::http_client::HttpClient;
use crate::model::{
    LanguageModel, LanguageModelProvider, LanguageModelProviderId, LanguageModelProviderName,
};
use crate::models::openai_provider::openai_model::{
     OPEN_AI_PROVIDER_ID, OPEN_AI_PROVIDER_NAME, OpenAiLanguageModel,
};
use std::sync::{Arc, Mutex};
use crate::openai;

const PROVIDER_ID: LanguageModelProviderId = OPEN_AI_PROVIDER_ID;
const PROVIDER_NAME: LanguageModelProviderName = OPEN_AI_PROVIDER_NAME;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct OpenAiSettings {
    pub api_url: String,
    pub api_key: String,
}
pub struct OpenAiLanguageModelProvider {
    http_client: Arc<dyn HttpClient>,
}

impl OpenAiLanguageModelProvider {
    pub fn new(client: Arc<dyn HttpClient>) -> Self {
        Self {
            http_client: client,
            // state: State::new(),
        }
    }
    pub fn create_language_model(&self, model: openai::Model) -> Arc<dyn LanguageModel> {
        Arc::new(OpenAiLanguageModel {
            id: model.id().to_string().into(),
            model,
            http_client: self.http_client.clone(),
            // state: self.state.clone(),
        })
    }
}
#[async_trait::async_trait]
impl LanguageModelProvider for OpenAiLanguageModelProvider {
    fn id(&self) -> LanguageModelProviderId {
        PROVIDER_ID
    }

    fn name(&self) -> LanguageModelProviderName {
        PROVIDER_NAME
    }

    fn default_model(&self) -> Option<Arc<dyn LanguageModel>> {
        todo!()
    }

    fn provided_models(&self) -> Vec<Arc<dyn LanguageModel>> {
        todo!()
    }
    fn default_fast_model(&self) -> Option<Arc<dyn LanguageModel>> {
        todo!()
    }

    // async fn authenticate(&self) -> anyhow::Result<()> {
    //     self.state.authenticate().await?;
    //     Ok(())
    // }
}
