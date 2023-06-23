use chrono::NaiveDateTime;
use humansize::{format_size_i, DECIMAL};
use tabled::builder::Builder;
use tabled::{row, Style, Table};

use crate::api::node::{NodeInfo, NodeInfoType};
use crate::commands::node::output::ValueFormatter;
use crate::errors::AnyError;

pub(crate) struct DefaultFormatter;

impl ValueFormatter for DefaultFormatter {
    fn format(
        &self,
        content: &NodeInfo,
        types: Option<&[NodeInfoType]>,
    ) -> Result<String, AnyError> {
        Ok(new_default_table(content, types).to_string())
    }
}

fn create_info_table(node_info: &NodeInfo) -> Table {
    let mut builder = Builder::default();
    builder.set_columns(vec![
        "ID".to_string(),
        "NAME".to_string(),
        "HOST".to_string(),
        "VERSION".to_string(),
        "HTTP_ADDRESS".to_string(),
        "STATUS".to_string(),
        "WORKERS".to_string(),
        "BATCH_SIZE".to_string(),
        "BATCH_DELAY".to_string(),
        "EPHEMERAL_ID".to_string(),
    ]);

    let node = &node_info.node;
    builder.add_record(vec![
        node.id.to_string(),
        node.name.to_string(),
        node.host.to_string(),
        node.version.to_string(),
        node.http_address.to_string(),
        node.status.to_string(),
        node.pipeline.workers.to_string(),
        node.pipeline.batch_size.to_string(),
        node.pipeline.batch_delay.to_string(),
        node.ephemeral_id.to_string(),
    ]);

    let mut table = builder.build();
    table.with(Style::empty());
    table
}

fn create_pipelines_table(node_info: &NodeInfo) -> Table {
    let mut builder = Builder::default();

    builder.set_columns(vec![
        "NAME".to_string(),
        "WORKERS".to_string(),
        "BATCH_SIZE".to_string(),
        "BATCH_DELAY".to_string(),
        "CONFIG_RELOAD_AUTOMATIC".to_string(),
        "CONFIG_RELOAD_INTERVAL".to_string(),
        "DLQ_ENABLED".to_string(),
        "EPHEMERAL_ID".to_string(),
    ]);

    if node_info.pipelines.is_some() {
        for (name, pipeline) in node_info.pipelines.as_ref().unwrap() {
            builder.add_record(vec![
                name.to_string(),
                pipeline.workers.to_string(),
                pipeline.batch_size.to_string(),
                pipeline.batch_delay.to_string(),
                pipeline.config_reload_automatic.to_string(),
                pipeline.config_reload_interval.to_string(),
                pipeline.dead_letter_queue_enabled.to_string(),
                pipeline.ephemeral_id.to_string(),
            ]);
        }
    }

    let mut table = builder.build();
    table.with(Style::empty());
    table
}

fn create_os_table(node_info: &NodeInfo) -> Table {
    let mut builder = Builder::default();

    builder.set_columns(vec![
        "NAME".to_string(),
        "VERSION".to_string(),
        "ARCH".to_string(),
        "AVAILABLE_PROCESSORS".to_string(),
    ]);

    if node_info.os.is_some() {
        let os = node_info.os.as_ref().unwrap();
        builder.add_record(vec![
            os.name.to_string(),
            os.version.to_string(),
            os.arch.to_string(),
            os.available_processors.to_string(),
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    table
}

fn create_jvm_table(node_info: &NodeInfo) -> Table {
    let mut builder = Builder::default();

    builder.set_columns(vec![
        "PID".to_string(),
        "VERSION".to_string(),
        "VM".to_string(),
        "START_TIME".to_string(),
        "HEAP_INIT_MAX".to_string(),
        "NON_HEAP_INIT_MAX".to_string(),
        "GC_COLLECTORS".to_string(),
    ]);

    if node_info.jvm.is_some() {
        let jvm = node_info.jvm.as_ref().unwrap();
        let jvm_start_time = NaiveDateTime::from_timestamp_millis(jvm.start_time_in_millis)
            .map(|value| value.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default();

        builder.add_record(vec![
            jvm.pid.to_string(),
            jvm.version.to_string(),
            format!("{} {} ({})", jvm.vm_name, jvm.vm_vendor, jvm.vm_version),
            jvm_start_time,
            format!(
                "{} / {}",
                humanize_bytes(&jvm.mem.heap_init_in_bytes),
                humanize_bytes(&jvm.mem.heap_max_in_bytes)
            ),
            format!(
                "{} / {}",
                humanize_bytes(&jvm.mem.non_heap_init_in_bytes),
                humanize_bytes(&jvm.mem.non_heap_max_in_bytes)
            ),
            jvm.gc_collectors.join(", "),
        ]);
    }

    let mut table = builder.build();
    table.with(Style::empty());
    table
}

fn new_default_table(node_info: &NodeInfo, types: Option<&[NodeInfoType]>) -> Table {
    let mut builder = Builder::default();

    match types {
        None => {
            add_all_tables(&mut builder, node_info);
        }
        Some(info_types) => {
            if info_types.contains(&NodeInfoType::All) {
                add_all_tables(&mut builder, node_info);
            } else {
                for info_type in info_types {
                    match info_type {
                        NodeInfoType::Os => {
                            add_section_separator(&mut builder, info_types, info_type);
                            builder.add_record(vec![create_os_table(node_info).to_string()]);
                        }
                        NodeInfoType::Pipelines => {
                            add_section_separator(&mut builder, info_types, info_type);
                            builder.add_record(vec![create_pipelines_table(node_info).to_string()]);
                        }
                        NodeInfoType::Jvm => {
                            add_section_separator(&mut builder, info_types, info_type);
                            builder.add_record(vec![create_jvm_table(node_info).to_string()]);
                        }
                        _ => {
                            add_section_separator(&mut builder, info_types, info_type);
                            builder.add_record(vec![
                                "Default format not supported for this type".to_string()
                            ]);
                        }
                    }
                }
            }
        }
    }

    let mut table = builder.build();
    table.with(Style::blank());
    table
}

fn add_section_separator(
    builder: &mut Builder,
    info_types: &[NodeInfoType],
    current: &NodeInfoType,
) {
    if info_types.len() > 1 {
        add_section_separator_record(builder, current);
    }
}

fn add_section_separator_record(builder: &mut Builder, current: &NodeInfoType) {
    let mut section = row![current.to_string().to_uppercase()];
    section.with(Style::empty());
    builder.add_record(vec![section.to_string()]);
}

fn add_all_tables(builder: &mut Builder, node_info: &NodeInfo) {
    add_section_separator_record(builder, &NodeInfoType::All);
    builder.add_record(vec![create_info_table(node_info).to_string()]);

    add_section_separator_record(builder, &NodeInfoType::Pipelines);
    builder.add_record(vec![create_pipelines_table(node_info).to_string()]);

    add_section_separator_record(builder, &NodeInfoType::Jvm);
    builder.add_record(vec![create_jvm_table(node_info).to_string()]);

    add_section_separator_record(builder, &NodeInfoType::Os);
    builder.add_record(vec![create_os_table(node_info).to_string()]);
}

fn humanize_bytes(b: &i64) -> String {
    format_size_i(*b, DECIMAL)
}
