use crate::api::Client;
use crate::api::node::{NodeInfo, NodeInfoType};
use crate::result::GenericResult;

impl Client {
    pub fn get_node_info_into_string(&self, types: &[NodeInfoType], _args: &[&str]) -> GenericResult<String> {
        let response = self.request("GET", &self.node_info_request_path(types), None)?;
        Ok(response.into_string()?)
    }

    pub fn get_node_info(&self, types: &[NodeInfoType], _args: &[&str]) -> GenericResult<NodeInfo> {
        let response = self.request("GET", &self.node_info_request_path(types), None)?;
        let node_info: NodeInfo = response.into_json()?;
        Ok(node_info)
    }

    fn node_info_request_path(&self, types: &[NodeInfoType]) -> String {
        let filterable_types = types.iter()
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