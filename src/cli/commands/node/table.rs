use json_to_table::json_to_table;
use serde_json::Value;
use tabled::Style;

use crate::api::node::{NodeInfo, NodeInfoType};
use crate::commands::node::json::remove_unlisted_fields;
use crate::commands::node::output::ValueFormatter;
use crate::result::GenericResult;

pub(crate) struct TableFormatter;

impl ValueFormatter for TableFormatter {
    fn format(&self, content: &NodeInfo, types: Option<&[NodeInfoType]>) -> GenericResult<String> {
        let value = serde_json::to_value(content)?;
        self.format_value(value, types)
    }

    fn format_value(&self, content: Value, types: Option<&[NodeInfoType]>) -> GenericResult<String> {
        let formatted_content = match types {
            None => content,
            Some(values) => remove_unlisted_fields(content, values)?
        };

        Ok(json_to_table(&formatted_content).set_style(Style::modern()).to_string())
    }
}