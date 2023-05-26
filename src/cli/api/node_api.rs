use crate::api::node::{NodeInfo, NodeInfoType};
use crate::api::stats::NodeStats;
use crate::api::Client;
use crate::errors::AnyError;
use serde_json::Value;

impl Client<'_> {
    pub const QUERY_NODE_INFO_GRAPH: &'static [(&'static str, &'static str)] = &[("graph", "true")];
    pub const QUERY_NODE_STATS_VERTICES: &'static [(&'static str, &'static str)] =
        &[("vertices", "true")];

    pub fn get_node_info_as_string(
        &self,
        types: &[NodeInfoType],
        query: Option<&[(&str, &str)]>,
    ) -> Result<String, AnyError> {
        let response = self.request("GET", &self.node_info_request_path(types), query)?;
        Ok(response.into_string()?)
    }

    pub fn get_node_info_as_value(
        &self,
        types: &[NodeInfoType],
        query: Option<&[(&str, &str)]>,
    ) -> Result<Value, AnyError> {
        let response = self.request("GET", &self.node_info_request_path(types), query)?;
        let value: Value = response.into_json()?;
        Ok(value)
    }

    pub fn get_node_info(
        &self,
        types: &[NodeInfoType],
        query: Option<&[(&str, &str)]>,
    ) -> Result<NodeInfo, AnyError> {
        let response = self.request("GET", &self.node_info_request_path(types), query)?;
        let node_info: NodeInfo = response.into_json()?;
        Ok(node_info)
    }

    pub fn get_node_stats(&self, query: Option<&[(&str, &str)]>) -> Result<NodeStats, AnyError> {
        let response = self.request("GET", &self.node_request_path("stats"), query)?;
        let node_stats: NodeStats = response.into_json()?;
        Ok(node_stats)
    }

    fn node_info_request_path(&self, types: &[NodeInfoType]) -> String {
        let filterable_types = types
            .iter()
            .filter(|info_type| **info_type != NodeInfoType::All)
            .map(|info_type| info_type.as_api_value())
            .collect::<Vec<&str>>();

        if filterable_types.len() != types.len() {
            return self.node_request_path(NodeInfoType::All.as_api_value());
        }

        let filters = filterable_types.join(",");
        self.node_request_path(&filters)
    }

    fn node_request_path(&self, request_path: &str) -> String {
        if request_path.is_empty() {
            return "_node".to_string();
        }

        format!("{}/{}", "_node", request_path)
    }
}
