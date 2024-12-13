use serde_json::Value;

use crate::api::node::{NodeInfo, NodeInfoType};
use crate::commands::node::json::JsonFormatter;
use crate::errors::AnyError;

pub trait ValueFormatter {
    fn format(
        &self,
        content: &NodeInfo,
        types: Option<&[NodeInfoType]>,
    ) -> Result<String, AnyError>;

    fn format_value(
        &self,
        content: Value,
        types: Option<&[NodeInfoType]>,
    ) -> Result<String, AnyError> {
        let node_info: NodeInfo = serde_json::from_value(content)?;
        Self::format(self, &node_info, types)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OutputFormat {
    Raw,
    Json,
}

impl TryFrom<&str> for OutputFormat {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "json" => Ok(OutputFormat::Json),
            "raw" => Ok(OutputFormat::Raw),
            _ => Err(format!("Invalid output format: {}!", value)),
        }
    }
}

impl OutputFormat {
    pub fn new_formatter(&self) -> Box<dyn ValueFormatter> {
        match self {
            OutputFormat::Json => Box::new(JsonFormatter {}),
            _ => Box::new(JsonFormatter {}),
        }
    }
}
