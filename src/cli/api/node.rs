use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Node {
    pub host: String,
    pub version: String,
    pub http_address: String,
    pub id: String,
    pub name: String,
    pub ephemeral_id: String,
    pub status: String,
    pub snapshot: Option<bool>,
    pub pipeline: PipelineSettings,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NodeInfo {
    #[serde(flatten)]
    pub node: Node,
    pub pipelines: Option<HashMap<String, PipelineInfo>>,
    pub os: Option<Os>,
    pub jvm: Option<Jvm>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Graph {
    pub graph: GraphDefinition,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GraphDefinition {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Edge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub r#type: String,
    pub when: Option<bool>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Vertex {
    pub id: String,
    pub explicit_id: bool,
    pub config_name: String,
    pub plugin_type: String,
    pub condition: String,
    pub r#type: String,
    pub meta: Option<VertexMeta>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VertexMeta {
    pub source: VertexMetaSource,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VertexMetaSource {
    pub protocol: String,
    pub id: String,
    pub line: u32,
    pub column: u32,
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineInfo {
    pub ephemeral_id: String,
    pub hash: String,
    pub workers: i64,
    pub batch_size: i64,
    pub batch_delay: i64,
    pub config_reload_automatic: bool,
    pub config_reload_interval: i64,
    pub dead_letter_queue_enabled: bool,
    pub graph: Graph,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Os {
    pub name: String,
    pub arch: String,
    pub version: String,
    pub available_processors: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Jvm {
    pub pid: i64,
    pub version: String,
    pub vm_version: String,
    pub vm_vendor: String,
    pub vm_name: String,
    pub start_time_in_millis: i64,
    pub mem: JvmMem,
    pub gc_collectors: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct JvmMem {
    pub heap_init_in_bytes: i64,
    pub heap_max_in_bytes: i64,
    pub non_heap_init_in_bytes: i64,
    pub non_heap_max_in_bytes: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineSettings {
    pub workers: i64,
    pub batch_size: i64,
    pub batch_delay: i64,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum NodeInfoType {
    All,
    Pipelines,
    Os,
    Jvm,
}

impl fmt::Display for NodeInfoType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl NodeInfoType {
    pub(crate) fn as_api_value(&self) -> &'static str {
        match self {
            NodeInfoType::Pipelines => "pipelines",
            NodeInfoType::Os => "os",
            NodeInfoType::Jvm => "jvm",
            NodeInfoType::All => "",
        }
    }
}

impl TryFrom<&str> for NodeInfoType {
    type Error = String;

    fn try_from(value: &str) -> Result<NodeInfoType, Self::Error> {
        let clean_value = value.to_lowercase().trim().to_string();

        match clean_value.as_str() {
            "pipelines" => Ok(NodeInfoType::Pipelines),
            "os" => Ok(NodeInfoType::Os),
            "jvm" => Ok(NodeInfoType::Jvm),
            "" => Ok(NodeInfoType::All),
            _ => Err(format!("Invalid info type: {}!", clean_value)),
        }
    }
}
