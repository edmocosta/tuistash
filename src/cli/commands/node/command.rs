use clap::Args;

use crate::api::node::NodeInfoType;
use crate::commands::node::output::OutputFormat;
use crate::commands::traits::RunnableCommand;
use crate::config::Config;
use crate::errors::AnyError;
use crate::output::Output;

#[derive(Args)]
pub struct NodeArgs {
    /// Valid values are 'node', 'os', 'jvm', 'pipelines' separated by comma
    #[arg()]
    pub types: Option<String>,

    #[arg(short)]
    pub output: Option<String>,
}

pub struct NodeCommand;

impl RunnableCommand<NodeArgs> for NodeCommand {
    fn run(&self, out: &mut Output, args: &NodeArgs, config: &Config) -> Result<(), AnyError> {
        let output_format = match &args.output {
            None => OutputFormat::Default,
            Some(value) => OutputFormat::try_from(value.as_ref())?,
        };

        let info_types = &NodeCommand::parse_info_types(&args.types)?;
        if output_format == OutputFormat::Raw {
            let raw = config.api.get_node_info_as_string(info_types, None)?;
            NodeCommand::write(out, raw.as_bytes())?;
        } else {
            let node_info = config.api.get_node_info_as_value(info_types, None)?;
            NodeCommand::write(
                out,
                output_format
                    .new_formatter()
                    .format_value(node_info, Some(info_types))?
                    .as_bytes(),
            )?;
        }

        Ok(())
    }
}

impl NodeCommand {
    fn write(out: &mut Output, buf: &[u8]) -> Result<(), AnyError> {
        out.handle.write_all(buf)?;
        out.handle.write_all(b"\n")?;
        Ok(())
    }

    fn parse_info_types(types: &Option<String>) -> Result<Vec<NodeInfoType>, AnyError> {
        return match types {
            None => Ok(vec![NodeInfoType::All]),
            Some(values) => {
                let parts = values.trim().split(',');
                let mut result: Vec<NodeInfoType> = Vec::with_capacity(values.len());
                for info_type in parts {
                    result.push(NodeInfoType::try_from(info_type)?);
                }

                Ok(result)
            }
        };
    }
}
