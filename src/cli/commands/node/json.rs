use std::collections::HashSet;

use colored_json::ColorMode;
use serde_json::Value;

use crate::api::node::{NodeInfo, NodeInfoType};
use crate::commands::node::output::ValueFormatter;
use crate::errors::AnyError;

pub(crate) struct JsonFormatter;

impl ValueFormatter for JsonFormatter {
    fn format(
        &self,
        content: &NodeInfo,
        types: Option<&[NodeInfoType]>,
    ) -> Result<String, AnyError> {
        let value = serde_json::to_value(content)?;
        Self::format_value(self, value, types)
    }

    fn format_value(
        &self,
        content: Value,
        types: Option<&[NodeInfoType]>,
    ) -> Result<String, AnyError> {
        let formatted_content = match types {
            None => content,
            Some(values) => remove_unlisted_fields(content, values)?,
        };

        Ok(colored_json::to_colored_json(
            &formatted_content,
            ColorMode::On,
        )?)
    }
}

pub(crate) fn remove_unlisted_fields(
    content: Value,
    types: &[NodeInfoType],
) -> Result<Value, AnyError> {
    let mut inner_map = content.as_object().unwrap().to_owned();
    let mut types_set: HashSet<String> = HashSet::with_capacity(types.len());

    types
        .iter()
        .map(|v| v.as_api_value().to_string())
        .for_each(|value| {
            types_set.insert(value);
        });

    if types_set.contains(NodeInfoType::All.as_api_value()) {
        return Ok(Value::Object(inner_map));
    }

    let json_fields: Vec<String> = inner_map.keys().map(|k| k.to_string()).collect();
    for key in json_fields {
        if !types_set.contains(&key) {
            inner_map.remove(&key);
        }
    }

    Ok(Value::Object(inner_map))
}
