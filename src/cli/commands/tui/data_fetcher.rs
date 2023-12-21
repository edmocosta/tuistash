use crate::api::node::{NodeInfo, NodeInfoType};
use crate::api::stats::NodeStats;
use crate::api::Client;
use crate::errors::AnyError;

pub(crate) struct DataFetcher<'a> {
    api: &'a Client<'a>,
}

impl<'a> DataFetcher<'a> {
    pub fn new(api: &'a Client) -> DataFetcher<'a> {
        DataFetcher { api }
    }

    pub fn fetch_info(&self) -> Result<NodeInfo, AnyError> {
        self.api.get_node_info(
            &[NodeInfoType::Pipelines],
            Some(Client::QUERY_NODE_INFO_GRAPH),
        )
    }

    pub fn fetch_stats(&self) -> Result<NodeStats, AnyError> {
        self.api
            .get_node_stats(Some(Client::QUERY_NODE_STATS_VERTICES))
    }
}
