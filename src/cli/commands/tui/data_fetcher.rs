use std::fs;
use std::path::Path;

use crate::api::node::{NodeInfo, NodeInfoType};
use crate::api::stats::NodeStats;
use crate::api::Client;
use crate::errors::{AnyError, TuiError};

pub(crate) trait DataFetcher<'a> {
    fn fetch_info(&self) -> Result<NodeInfo, AnyError>;
    fn fetch_stats(&self) -> Result<NodeStats, AnyError>;
}

pub(crate) struct ApiDataFetcher<'a> {
    api: &'a Client<'a>,
}

impl<'a> ApiDataFetcher<'a> {
    pub fn new(api: &'a Client) -> ApiDataFetcher<'a> {
        ApiDataFetcher { api }
    }
}

impl<'a> DataFetcher<'a> for ApiDataFetcher<'a> {
    fn fetch_info(&self) -> Result<NodeInfo, AnyError> {
        self.api.get_node_info(
            &[NodeInfoType::Pipelines],
            Some(Client::QUERY_NODE_INFO_GRAPH),
        )
    }

    fn fetch_stats(&self) -> Result<NodeStats, AnyError> {
        self.api
            .get_node_stats(Some(Client::QUERY_NODE_STATS_VERTICES))
    }
}

pub(crate) struct PathDataFetcher {
    path: String,
}

const LOGSTASH_NODE_FILE: &str = "logstash_node.json";
const LOGSTASH_NODE_STATS_FILE: &str = "logstash_node_stats.json";
const LOGSTASH_DIAGNOSTIC_FILES: &[&str; 2] = &[LOGSTASH_NODE_FILE, LOGSTASH_NODE_STATS_FILE];

impl PathDataFetcher {
    fn validate_path(path: &str) -> Result<(), TuiError> {
        let mut missing_files = vec![];

        for file in LOGSTASH_DIAGNOSTIC_FILES {
            let file_path = Path::new(path).join(file);
            if !file_path.exists() || file_path.is_dir() {
                missing_files.push(file.to_string());
            }
        }

        if missing_files.is_empty() {
            Ok(())
        } else {
            let message = format!(
                "File(s) {} not found on the provided diagnostic path",
                missing_files.join(", ").as_str()
            );

            Err(TuiError::from(message.as_str()))
        }
    }
}

impl PathDataFetcher {
    pub fn new(path: String) -> Result<PathDataFetcher, AnyError> {
        if let Err(err) = PathDataFetcher::validate_path(&path) {
            Err(From::from(err))
        } else {
            Ok(PathDataFetcher { path })
        }
    }
}

impl<'a> DataFetcher<'a> for PathDataFetcher {
    fn fetch_info(&self) -> Result<NodeInfo, AnyError> {
        let path = Path::new(self.path.as_str()).join(LOGSTASH_NODE_FILE);
        return match fs::read_to_string(path) {
            Ok(data) => {
                let node_info: NodeInfo = serde_json::from_str(data.as_str()).unwrap();
                return Ok(node_info);
            }
            Err(err) => Err(err.into()),
        };
    }

    fn fetch_stats(&self) -> Result<NodeStats, AnyError> {
        let path = Path::new(self.path.as_str()).join(LOGSTASH_NODE_STATS_FILE);
        return match fs::read_to_string(path) {
            Ok(data) => {
                let node_stats: NodeStats = serde_json::from_str(data.as_str()).unwrap();
                return Ok(node_stats);
            }
            Err(err) => Err(err.into()),
        };
    }
}
