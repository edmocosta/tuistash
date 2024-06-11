use std::collections::HashMap;

use serde::Serialize;
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NodeStats {
    pub version: String,
    pub pipeline: PipelineDefaultSettings,
    pub jvm: Jvm,
    pub process: Process,
    pub events: Events,
    pub flow: Flow,
    pub pipelines: HashMap<String, PipelineStats>,
    pub reloads: Reloads,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct NodeStatsVertex {
    pub id: String,
    pub pipeline_ephemeral_id: String,
    pub events_out: i64,
    pub events_in: i64,
    pub duration_in_millis: u64,
    pub queue_push_duration_in_millis: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineDefaultSettings {
    pub workers: i64,
    pub batch_size: i64,
    pub batch_delay: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Jvm {
    pub threads: JvmThreads,
    pub mem: JvmMem,
    pub uptime_in_millis: u64,
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
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Process {
    pub open_file_descriptors: i64,
    pub peak_open_file_descriptors: i64,
    pub max_file_descriptors: i64,
    pub cpu: ProcessCpu,
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
    pub duration_in_millis: u64,
    pub queue_push_duration_in_millis: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Flow {
    pub input_throughput: FlowMetricValue,
    pub filter_throughput: FlowMetricValue,
    pub output_throughput: FlowMetricValue,
    pub queue_backpressure: FlowMetricValue,
    pub worker_concurrency: FlowMetricValue,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FlowMetricValue {
    #[serde(with = "infinity_f64_value")]
    pub current: f64,
    #[serde(with = "optional_infinity_f64_value")]
    pub last_1_minute: Option<f64>,
    #[serde(with = "optional_infinity_f64_value")]
    pub last_5_minutes: Option<f64>,
    #[serde(with = "optional_infinity_f64_value")]
    pub last_15_minutes: Option<f64>,
    #[serde(with = "optional_infinity_f64_value")]
    pub last_1_hour: Option<f64>,
    #[serde(with = "optional_infinity_f64_value")]
    pub last_24_hours: Option<f64>,
    #[serde(with = "infinity_f64_value")]
    pub lifetime: f64,
}

mod infinity_f64_value {
    use serde::de::{Deserialize, Deserializer};
    use serde::ser::Serializer;

    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(*value)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        return match f64::deserialize(deserializer) {
            Ok(v) => Ok(v),
            Err(_) => Ok(f64::INFINITY),
        };
    }
}

mod optional_infinity_f64_value {
    use serde::de::{Deserialize, Deserializer};
    use serde::ser::Serializer;

    pub fn serialize<S>(value: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(v) = value {
            serializer.serialize_f64(*v)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        return match f64::deserialize(deserializer) {
            Ok(v) => Ok(Some(v)),
            Err(_) => Ok(Some(f64::INFINITY)),
        };
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineStats {
    #[serde(deserialize_with = "deserialize_null_default")]
    pub events: Events,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub flow: PipelineFlow,
    pub plugins: Plugins,
    pub reloads: Reloads,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub queue: Queue,
    #[serde(with = "vertices")]
    pub vertices: HashMap<String, NodeStatsVertex>,
    pub ephemeral_id: Option<String>,
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

mod vertices {
    use std::collections::HashMap;

    use serde::de::{Deserialize, Deserializer};
    use serde::ser::Serializer;

    use super::NodeStatsVertex;

    pub fn serialize<S>(
        map: &HashMap<String, NodeStatsVertex>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(map.values())
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<String, NodeStatsVertex>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vertices = Vec::<NodeStatsVertex>::deserialize(deserializer)?;
        let mut map = HashMap::with_capacity(vertices.len());

        for item in vertices {
            map.insert(item.id.to_string(), item);
        }

        Ok(map)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PipelineFlow {
    pub input_throughput: FlowMetricValue,
    pub filter_throughput: FlowMetricValue,
    pub output_throughput: FlowMetricValue,
    pub queue_backpressure: FlowMetricValue,
    pub worker_concurrency: FlowMetricValue,
    pub queue_persisted_growth_bytes: FlowMetricValue,
    pub queue_persisted_growth_events: FlowMetricValue,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Plugins {
    #[serde(with = "plugins")]
    pub inputs: HashMap<String, Plugin>,
    #[serde(with = "plugins")]
    pub filters: HashMap<String, Plugin>,
    #[serde(with = "plugins")]
    pub codecs: HashMap<String, Plugin>,
    #[serde(with = "plugins")]
    pub outputs: HashMap<String, Plugin>,
}

impl Plugins {
    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Option<&Plugin> {
        if let Some(plugin) = self.inputs.get(id) {
            return Some(plugin);
        }

        if let Some(plugin) = self.filters.get(id) {
            return Some(plugin);
        }

        if let Some(plugin) = self.outputs.get(id) {
            return Some(plugin);
        }

        return self.codecs.get(id);
    }

    pub fn all(&self) -> HashMap<String, &Plugin> {
        self.all_with_type()
            .iter()
            .map(|(id, plugin)| (id.to_string(), plugin.1))
            .collect()
    }

    pub fn all_with_type(&self) -> HashMap<String, (String, &Plugin)> {
        let mut map = HashMap::with_capacity(
            self.inputs.len() + self.filters.len() + self.outputs.len() + self.codecs.len(),
        );

        for (id, plugin) in &self.inputs {
            map.insert(id.to_string(), ("input".to_string(), plugin));
        }

        for (id, plugin) in &self.codecs {
            map.insert(id.to_string(), ("codec".to_string(), plugin));
        }

        for (id, plugin) in &self.filters {
            map.insert(id.to_string(), ("filter".to_string(), plugin));
        }

        for (id, plugin) in &self.outputs {
            map.insert(id.to_string(), ("output".to_string(), plugin));
        }

        map
    }

    pub fn avg_duration_in_millis_percentage(&self, pipeline_duration_in_millis: u64) -> f64 {
        let all_plugins = self.all();
        let duration_in_millis_sum: f64 = all_plugins
            .values()
            .map(|plugin| {
                (plugin.events.duration_in_millis as f64 / pipeline_duration_in_millis as f64)
                    * 100.0
            })
            .sum();

        duration_in_millis_sum / (all_plugins.len() as f64)
    }
}

mod plugins {
    use std::collections::HashMap;

    use serde::de::{Deserialize, Deserializer};
    use serde::ser::Serializer;

    use crate::api::stats::Plugin;

    pub fn serialize<S>(map: &HashMap<String, Plugin>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(map.values())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, Plugin>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = HashMap::new();
        for item in Vec::<Plugin>::deserialize(deserializer)? {
            map.insert(item.id.to_string(), item);
        }

        Ok(map)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Plugin {
    pub id: String,
    pub name: Option<String>,
    pub flow: Option<PluginFlow>,
    pub events: Events,
    #[serde(flatten)]
    pub other: Map<String, Value>,
}

impl Plugin {
    pub fn get_other<'a, U>(&'a self, field: &str, mapper: fn(&'a Value) -> U, default: U) -> U {
        let mut split: Vec<&str> = field.split('.').rev().collect();
        let mut current: &Map<String, Value> = &self.other;

        while let Some(path) = split.pop() {
            if let Some(value) = current.get(path) {
                if split.is_empty() {
                    return mapper(value);
                }
                if let Some(map_value) = value.as_object() {
                    current = map_value;
                } else {
                    return default;
                }
            } else {
                return default;
            }
        }

        default
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PluginFlow {
    pub throughput: Option<FlowMetricValue>,
    pub worker_utilization: Option<FlowMetricValue>,
    pub worker_millis_per_event: Option<FlowMetricValue>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Reloads {
    pub successes: i64,
    pub failures: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Queue {
    #[serde(rename = "type")]
    pub r#type: String,
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
