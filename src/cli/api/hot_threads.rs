use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NodeHotThreads {
    pub hot_threads: HotThreads,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HotThreads {
    pub time: String,
    pub busiest_threads: u64,
    #[serde(with = "threads")]
    pub threads: HashMap<i64, Thread>,
}

mod threads {
    use std::collections::HashMap;

    use serde::de::{Deserialize, Deserializer};
    use serde::ser::Serializer;

    use super::Thread;

    pub fn serialize<S>(map: &HashMap<i64, Thread>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(map.values())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<i64, Thread>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vertices = Vec::<Thread>::deserialize(deserializer)?;
        let mut map = HashMap::with_capacity(vertices.len());

        for item in vertices {
            map.insert(item.thread_id, item);
        }

        Ok(map)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Thread {
    pub name: String,
    pub thread_id: i64,
    pub percent_of_cpu_time: f64,
    pub state: String,
    pub traces: Vec<String>,
}
