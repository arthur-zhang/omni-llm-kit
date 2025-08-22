mod model;
mod errors;
mod types;
mod language_provider;
mod request;
mod rate_limiter;

pub use types::*;
pub use language_provider::*;
pub use model::*;
pub use request::*;
pub use errors::*;


pub const ANTHROPIC_PROVIDER_ID: LanguageModelProviderId =
    LanguageModelProviderId::new("anthropic");
pub const ANTHROPIC_PROVIDER_NAME: LanguageModelProviderName =
    LanguageModelProviderName::new("Anthropic");

pub const GOOGLE_PROVIDER_ID: LanguageModelProviderId = LanguageModelProviderId::new("google");
pub const GOOGLE_PROVIDER_NAME: LanguageModelProviderName =
    LanguageModelProviderName::new("Google AI");

pub const OPEN_AI_PROVIDER_ID: LanguageModelProviderId = LanguageModelProviderId::new("openai");
pub const OPEN_AI_PROVIDER_NAME: LanguageModelProviderName =
    LanguageModelProviderName::new("OpenAI");