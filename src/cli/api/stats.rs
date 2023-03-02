use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NodeStats {
    pub host: String,
    pub version: String,
    pub http_address: String,
    pub id: String,
    pub name: String,
    pub ephemeral_id: String,
    pub status: String,
    pub snapshot: bool,
    pub pipeline: PipelineSettings,
    pub jvm: Jvm,
    pub process: Process,
    pub events: Events,
    pub flow: Flow,
    pub pipelines: HashMap<String, Pipeline>,
    pub reloads: ReloadsStat,
    pub os: Os,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineSettings {
    pub workers: i64,
    pub batch_size: i64,
    pub batch_delay: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Jvm {
    pub threads: JvmThreads,
    pub mem: JvmMem,
    pub gc: JvmGc,
    pub uptime_in_millis: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct JvmThreads {
    pub count: i64,
    pub peak_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct JvmMem {
    pub heap_used_percent: i64,
    pub heap_committed_in_bytes: i64,
    pub heap_max_in_bytes: i64,
    pub heap_used_in_bytes: i64,
    pub non_heap_used_in_bytes: i64,
    pub non_heap_committed_in_bytes: i64,
    pub pools: JvmMemPools,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct JvmMemPools {
    pub survivor: JvmMemPoolStat,
    pub old: JvmMemPoolStat,
    pub young: JvmMemPoolStat,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct JvmMemPoolStat {
    pub peak_used_in_bytes: i64,
    pub used_in_bytes: i64,
    pub peak_max_in_bytes: i64,
    pub max_in_bytes: i64,
    pub committed_in_bytes: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct JvmGc {
    pub collectors: JvmGcCollectors,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct JvmGcCollectors {
    pub old: JvmGcCollectorStats,
    pub young: JvmGcCollectorStats,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct JvmGcCollectorStats {
    pub collection_time_in_millis: i64,
    pub collection_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Process {
    pub open_file_descriptors: i64,
    pub peak_open_file_descriptors: i64,
    pub max_file_descriptors: i64,
    pub mem: ProcessMem,
    pub cpu: ProcessCpu,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessMem {
    pub total_virtual_in_bytes: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessCpu {
    pub total_in_millis: i64,
    pub percent: i64,
    pub load_average: ProcessCpuLoadAverage,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProcessCpuLoadAverage {
    #[serde(rename = "1m")]
    pub l1m: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Events {
    #[serde(rename = "in")]
    pub r#in: i64,
    pub filtered: i64,
    pub out: i64,
    pub duration_in_millis: i64,
    pub queue_push_duration_in_millis: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Flow {
    pub input_throughput: MetricValue,
    pub filter_throughput: MetricValue,
    pub output_throughput: MetricValue,
    pub queue_backpressure: MetricValue,
    pub worker_concurrency: MetricValue,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MetricValue {
    pub current: f64,
    pub lifetime: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pipeline {
    pub events: PipelineEvents,
    pub flow: PipelineFlow,
    pub plugins: Plugins,
    pub reloads: Reloads,
    pub queue: Queue,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineEvents {
    pub duration_in_millis: i64,
    #[serde(rename = "in")]
    pub input: i64,
    pub filtered: i64,
    pub out: i64,
    pub queue_push_duration_in_millis: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineFlow {
    pub input_throughput: MetricValue,
    pub filter_throughput: MetricValue,
    pub output_throughput: MetricValue,
    pub queue_backpressure: MetricValue,
    pub worker_concurrency: MetricValue,
    pub queue_persisted_growth_bytes: MetricValue,
    pub queue_persisted_growth_events: MetricValue,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Plugins {
    pub inputs: Vec<InputPlugin>,
    pub filters: Vec<FilterPlugin>,
    pub outputs: Vec<OutputPlugin>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputPlugin {
    pub id: String,
    pub events: InputPluginEvents,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputPluginEvents {
    pub out: i64,
    pub queue_push_duration_in_millis: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterPlugin {
    pub id: String,
    pub events: FilterPluginEvents,
    pub failures: Option<i64>,
    pub patterns_per_field: Option<PatternsPerField>,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterPluginEvents {
    pub duration_in_millis: i64,
    #[serde(rename = "in")]
    pub in_field: i64,
    pub out: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PatternsPerField {
    pub message: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputPlugin {
    pub id: String,
    pub events: OutputPluginEvents,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputPluginEvents {
    pub duration_in_millis: i64,
    #[serde(rename = "in")]
    pub in_field: i64,
    pub out: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Reloads {
    pub last_error: Value,
    pub successes: i64,
    pub last_success_timestamp: Value,
    pub last_failure_timestamp: Value,
    pub failures: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Queue {
    #[serde(rename = "type")]
    pub type_field: String,
    pub capacity: QueueCapacity,
    pub data: QueueData,
    pub events: i64,
    pub events_count: i64,
    pub queue_size_in_bytes: i64,
    pub max_queue_size_in_bytes: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct QueueCapacity {
    pub max_unread_events: i64,
    pub page_capacity_in_bytes: i64,
    pub max_queue_size_in_bytes: i64,
    pub queue_size_in_bytes: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct QueueData {
    pub path: String,
    pub free_space_in_bytes: i64,
    pub storage_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReloadsStat {
    pub successes: i64,
    pub failures: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Os {
    pub cgroup: OsCgroup,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OsCgroup {
    pub cpuacct: OsCgroupCpuacct,
    pub cpu: OsCgroupCpu,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OsCgroupCpuacct {
    pub control_group: String,
    pub usage_nanos: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OsCgroupCpu {
    pub control_group: String,
    pub cfs_period_micros: i64,
    pub cfs_quota_micros: i64,
    pub stat: OsCgroupCpuStat,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OsCgroupCpuStat {
    pub number_of_elapsed_periods: i64,
    pub number_of_times_throttled: i64,
    pub time_throttled_nanos: i64,
}
