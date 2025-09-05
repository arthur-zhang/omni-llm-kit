use crate::common::SharedString;
use crate::{
    LanguageModel, LanguageModelImage, LanguageModelRequest, LanguageModelToolSchemaFormat,
};
use async_trait::async_trait;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum ToolSource {
    /// A native tool built-in to Zed.
    Native,
    /// A tool provided by a context server.
    ContextServer { id: String },
}
/// A tool that can be used by a language model.
pub trait Tool: Sized + Send + Sync {
    const NAME: &'static str;
    /// Returns the name of the tool.
    fn name(&self) -> String {
        Self::NAME.to_string()
    }

    /// Returns the description of the tool.
    fn description(&self) -> String;

    /// Returns the source of the tool.
    fn source(&self) -> ToolSource {
        ToolSource::Native
    }

    /// Returns true if the tool needs the users's confirmation
    /// before having permission to run.
    fn needs_confirmation(&self, input: &serde_json::Value) -> bool;

    /// Returns true if the tool may perform edits.
    fn may_perform_edits(&self) -> bool;

    /// Returns the JSON schema that describes the tool's input.
    fn input_schema(&self, _: LanguageModelToolSchemaFormat) -> anyhow::Result<serde_json::Value> {
        Ok(serde_json::Value::Object(serde_json::Map::default()))
    }
    fn ui_text(&self, input: &serde_json::Value) -> String;
    /// Runs the tool with the provided input.
    fn run(
        &self,
        input: serde_json::Value,
    ) -> impl Future<Output = anyhow::Result<ToolResultContent>> + Send;
}
pub trait ToolDyn: Send + Sync {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn input_schema(&self, _: LanguageModelToolSchemaFormat) -> anyhow::Result<serde_json::Value>;
    fn needs_confirmation(&self, input: &serde_json::Value) -> bool;
    fn ui_text(&self, input: &serde_json::Value) -> String;
    fn run(
        &self,
        input: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ToolResultContent>> + Send + '_>>;
}
impl<T: Tool> ToolDyn for T {
    fn name(&self) -> String {
        self.name()
    }

    fn description(&self) -> String {
        self.description()
    }
    fn input_schema(
        &self,
        schema: LanguageModelToolSchemaFormat,
    ) -> anyhow::Result<serde_json::Value> {
        self.input_schema(schema)
    }
    fn ui_text(&self, input: &serde_json::Value) -> String {
        self.ui_text(input)
    }

    fn run(
        &self,
        input: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ToolResultContent>> + Send + '_>> {
        Box::pin(async move { <Self as Tool>::run(self, input).await })
    }

    fn needs_confirmation(&self, input: &serde_json::Value) -> bool {
        self.needs_confirmation(input)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ToolResultContent {
    Text(String),
    Image(LanguageModelImage),
}

impl ToolResultContent {
    pub fn len(&self) -> usize {
        match self {
            ToolResultContent::Text(str) => str.len(),
            ToolResultContent::Image(image) => image.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            ToolResultContent::Text(str) => str.is_empty(),
            ToolResultContent::Image(image) => image.is_empty(),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            ToolResultContent::Text(str) => Some(str),
            ToolResultContent::Image(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ToolUseStatus {
    InputStillStreaming,
    NeedsConfirmation,
    Pending,
    Running,
    Finished(SharedString),
    Error(SharedString),
}

impl ToolUseStatus {
    pub fn text(&self) -> SharedString {
        match self {
            ToolUseStatus::NeedsConfirmation => "".into(),
            ToolUseStatus::InputStillStreaming => "".into(),
            ToolUseStatus::Pending => "".into(),
            ToolUseStatus::Running => "".into(),
            ToolUseStatus::Finished(out) => out.clone(),
            ToolUseStatus::Error(out) => out.clone(),
        }
    }

    pub fn error(&self) -> Option<SharedString> {
        match self {
            ToolUseStatus::Error(out) => Some(out.clone()),
            _ => None,
        }
    }
}
