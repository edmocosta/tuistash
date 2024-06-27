use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use regex::{Captures, Regex, RegexBuilder};

use crate::api::hot_threads::{HotThreads, NodeHotThreads, Thread};
use crate::api::node::{NodeInfo, NodeInfoType};
use crate::api::stats::NodeStats;
use crate::api::Client;
use crate::errors::{AnyError, TuiError};

pub(crate) trait DataFetcher<'a> {
    fn fetch_info(&self) -> Result<NodeInfo, AnyError>;
    fn fetch_stats(&self) -> Result<NodeStats, AnyError>;
    fn fetch_hot_threads(&self) -> Result<NodeHotThreads, AnyError>;
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

    fn fetch_hot_threads(&self) -> Result<NodeHotThreads, AnyError> {
        self.api.get_hot_threads(Some(&[
            ("threads", "500"),
            ("ignore_idle_threads", "false"),
        ]))
    }
}

pub(crate) struct PathDataFetcher {
    path: String,
}

const LOGSTASH_NODE_FILE: &str = "logstash_node.json";
const LOGSTASH_NODE_STATS_FILE: &str = "logstash_node_stats.json";
const LOGSTASH_NODE_HOT_THREADS_FILE: &str = "logstash_nodes_hot_threads.json";
const LOGSTASH_DIAGNOSTIC_FILES: &[&str; 3] = &[
    LOGSTASH_NODE_FILE,
    LOGSTASH_NODE_STATS_FILE,
    LOGSTASH_NODE_HOT_THREADS_FILE,
];

impl PathDataFetcher {
    pub fn new(path: String) -> Result<PathDataFetcher, AnyError> {
        if let Err(err) = PathDataFetcher::validate_path(&path) {
            Err(From::from(err))
        } else {
            Ok(PathDataFetcher { path })
        }
    }

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

    fn parse_hot_threads_human_file(&self) -> Result<NodeHotThreads, AnyError> {
        let file = File::open(Path::new(self.path.as_str()).join(LOGSTASH_NODE_HOT_THREADS_FILE))?;
        let file_buffer = BufReader::new(file);

        let mut hot_threads: HotThreads = HotThreads::default();
        let header_regex =
            Regex::new(r"Hot threads at (?<time>\w.*), busiestThreads=(?<threads>\d.*):")?;
        let mut file_buffer_lines = file_buffer.lines().skip(1);

        if let Some(header_line) = file_buffer_lines.next() {
            if let Ok(header_line_content) = header_line {
                let header_captures = header_regex.captures(&header_line_content);

                if header_captures.is_none() {
                    return Ok(NodeHotThreads::default());
                }

                let captures = header_captures.unwrap();
                hot_threads.time = Self::get_captured_group_string("time", &captures);
                hot_threads.busiest_threads =
                    Self::get_captured_group_integer("threads", &captures) as u64;
            } else {
                // Ignore malformed files
                return Ok(NodeHotThreads { hot_threads });
            }
        }

        let threads_regex = RegexBuilder::new(r"((?<usage>[+-]?(?:[0-9]*[.])?[0-9]+) % of cpu usage, state: (?<state>.*), thread name: '(?<name>.*)', thread id: (?<id>\d.*) (?<traces>(?:.|\n)*?)-{5,}\n)")
            .multi_line(true)
            .build()?;

        let mut all_thread_lines: String = String::new();
        for line in file_buffer_lines {
            let line_content = &line.unwrap_or_default();
            all_thread_lines.push_str(&format!("{}\n", line_content));
        }

        let captures = threads_regex.captures_iter(&all_thread_lines);
        for thread_capture in captures {
            let id = Self::get_captured_group_integer("id", &thread_capture);
            let usage = Self::get_captured_group_float("usage", &thread_capture);
            let name = Self::get_captured_group_string("name", &thread_capture);
            let state = Self::get_captured_group_string("state", &thread_capture);
            let traces = Self::get_captured_group_string("traces", &thread_capture)
                .split('\n')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect();

            hot_threads.threads.insert(
                id,
                Thread {
                    name: name.to_string(),
                    thread_id: id,
                    percent_of_cpu_time: usage,
                    state: state.to_string(),
                    traces,
                },
            );
        }

        Ok(NodeHotThreads { hot_threads })
    }

    fn get_captured_group_string(name: &str, captures: &Captures) -> String {
        captures
            .name(name)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default()
    }

    fn get_captured_group_integer(name: &str, captures: &Captures) -> i64 {
        captures
            .name(name)
            .map(|m| m.as_str().parse::<i64>().unwrap_or_default())
            .unwrap_or_default()
    }

    fn get_captured_group_float(name: &str, captures: &Captures) -> f64 {
        captures
            .name(name)
            .map(|m| m.as_str().parse::<f64>().unwrap_or_default())
            .unwrap_or_default()
    }
}

impl<'a> DataFetcher<'a> for PathDataFetcher {
    fn fetch_info(&self) -> Result<NodeInfo, AnyError> {
        let path = Path::new(self.path.as_str()).join(LOGSTASH_NODE_FILE);
        match fs::read_to_string(path) {
            Ok(data) => {
                let node_info: NodeInfo = serde_json::from_str(data.as_str())?;
                Ok(node_info)
            }
            Err(err) => Err(err.into()),
        }
    }

    fn fetch_stats(&self) -> Result<NodeStats, AnyError> {
        let path = Path::new(self.path.as_str()).join(LOGSTASH_NODE_STATS_FILE);
        match fs::read_to_string(path) {
            Ok(data) => {
                let node_stats: NodeStats = serde_json::from_str(data.as_str())?;
                Ok(node_stats)
            }
            Err(err) => Err(err.into()),
        }
    }

    fn fetch_hot_threads(&self) -> Result<NodeHotThreads, AnyError> {
        let json_file_path = Path::new(self.path.as_str()).join(LOGSTASH_NODE_HOT_THREADS_FILE);

        // Old versions of the Logstash diagnostic tool was generating the hot-threads file
        // using the human format instead of JSON.
        let mut read_as_json = false;
        if let Ok(mut file) = File::open(&json_file_path) {
            let mut first_byte = [0; 1];
            file.read_exact(&mut first_byte)?;
            read_as_json = first_byte[0] == b'{';
        }

        if read_as_json {
            return match fs::read_to_string(&json_file_path) {
                Ok(data) => {
                    let node_hot_threads: NodeHotThreads = serde_json::from_str(data.as_str())?;
                    return Ok(node_hot_threads);
                }
                Err(err) => Err(err.into()),
            };
        }

        self.parse_hot_threads_human_file()
    }
}
