use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct SessionState {
    pub(crate) session_id: String,
    pub(crate) active_tab_id: Option<String>,
    pub(crate) tabs: Vec<String>,
    pub(crate) port: Option<u16>,
    pub(crate) host: Option<String>,
    pub(crate) browser_name: Option<String>,
    pub(crate) ref_map: Option<HashMap<String, String>>,
    pub(crate) ref_map_url: Option<String>,
    pub(crate) ref_map_timestamp: Option<i64>,
    pub(crate) prev_elements: Option<Vec<InteractiveElement>>,
    pub(crate) current_elements: Option<Vec<InteractiveElement>>,
    pub(crate) active_frame_id: Option<String>,
    pub(crate) dialog_handler: Option<DialogHandler>,
    pub(crate) block_patterns: Option<BlockPatterns>,
    pub(crate) download_dir: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DebugTab {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) url: Option<String>,
    #[serde(default)]
    pub(crate) web_socket_debugger_url: Option<String>,
}

#[derive(Debug)]
pub(crate) struct BrowserCandidate {
    pub(crate) path: String,
    pub(crate) name: String,
}

#[derive(Debug)]
pub(crate) struct KeyModifiers {
    pub(crate) ctrl: bool,
    pub(crate) alt: bool,
    pub(crate) shift: bool,
    pub(crate) meta: bool,
}

#[derive(Debug)]
pub(crate) struct KeyMapping {
    pub(crate) key: String,
    pub(crate) code: String,
    pub(crate) key_code: i64,
}

#[derive(Debug)]
pub(crate) struct LocatedElement {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) tag: String,
    pub(crate) text: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct DialogHandler {
    pub(crate) accept: bool,
    pub(crate) prompt_text: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct BlockPatterns {
    pub(crate) resource_types: Vec<String>,
    pub(crate) url_patterns: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct InteractiveElement {
    #[serde(rename = "ref")]
    pub(crate) ref_id: usize,
    pub(crate) role: String,
    pub(crate) name: String,
    pub(crate) value: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ActionCacheEntry {
    pub(crate) ref_map: HashMap<String, String>,
    pub(crate) elements: Vec<InteractiveElement>,
    pub(crate) output: String,
    pub(crate) timestamp: i64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct TabLock {
    pub(crate) session_id: String,
    pub(crate) expires: i64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct NetworkRequestLog {
    pub(crate) id: String,
    pub(crate) method: String,
    pub(crate) url: String,
    pub(crate) req_type: String,
    pub(crate) time: i64,
    pub(crate) status: Option<i64>,
    pub(crate) status_text: Option<String>,
    pub(crate) mime_type: Option<String>,
    pub(crate) post_data: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CommandFileEntry {
    pub(crate) command: String,
    #[serde(default)]
    pub(crate) args: Vec<Value>,
}
