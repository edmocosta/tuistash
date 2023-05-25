use crate::api::node::{NodeInfo, NodeInfoType};
use crate::api::stats::NodeStats;
use crate::api::Client;
use crate::errors::AnyError;

impl Client {
    pub fn get_node_info_into_string(
        &self,
        types: &[NodeInfoType],
        _args: &[&str],
    ) -> Result<String, AnyError> {
        let response = self.request("GET", &self.node_info_request_path(types), None)?;
        Ok(response.into_string()?)
    }

    pub fn get_node_info(
        &self,
        types: &[NodeInfoType],
        _args: &[&str],
    ) -> Result<NodeInfo, AnyError> {
        let response = self.request(
            "GET",
            &self.node_info_request_path(types),
            Some(&[("graph".to_string(), "true".to_string())]),
        )?;
        let node_info: NodeInfo = response.into_json()?;
        Ok(node_info)
    }

    pub fn get_node_stats(&self) -> Result<NodeStats, AnyError> {
        let response = self.request(
            "GET",
            &self.node_request_path("stats"),
            Some(&[("vertices".to_string(), "true".to_string())]),
        )?;
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
        return self.node_request_path(&filters);
    }

    fn node_request_path(&self, request_path: &str) -> String {
        if request_path.is_empty() {
            return "_node".to_string();
        }

        return format!("{}/{}", "_node", request_path);
    }
}
