use crate::{Tool, ToolDyn};
use derive_more::{Deref, DerefMut};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::sync::Arc;

#[derive(Default)]
struct ToolRegistryState {
    tools: FxHashMap<Arc<str>, Arc<dyn ToolDyn>>,
}
#[derive(Default)]
pub struct ToolRegistry {
    state: RwLock<ToolRegistryState>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(ToolRegistryState {
                tools: FxHashMap::default(),
            }),
        }
    }
    pub fn tool(&self, name: &str) -> Option<Arc<dyn ToolDyn>> {
        self.state.read().tools.get(name).cloned()
    }
    pub fn register_tool(&self, tool: impl ToolDyn + 'static) {
        eprintln!("registering tool {:?}", tool.name());
        let mut state = self.state.write();
        let tool_name: Arc<str> = tool.name().into();
        state.tools.insert(tool_name, Arc::new(tool));
    }

    pub fn tools(&self) -> Vec<Arc<dyn ToolDyn>> {
        self.state.read().tools.values().cloned().collect()
    }
}
