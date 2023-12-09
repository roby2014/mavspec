mod cli {
    use clap::{Parser, Subcommand};

    const DEFAULT_OUT_PATH: &str = "";
    #[cfg(feature = "rust")]
    const DEFAULT_RUST_MODULE_PATH: &str = "mavlink";

    /// MavLink code generator.
    #[derive(Debug, Parser)]
    #[command(author, version, about, long_about = None)]
    pub struct Cli {
        #[command(subcommand)]
        pub command: Option<Commands>,

        /// List of sources
        #[arg(short, long)]
        pub src: Vec<String>,

        /// Path to output specs
        #[arg(short, long, default_value = DEFAULT_OUT_PATH)]
        pub out: String,
    }

    /// Cli commands
    #[derive(Debug, Subcommand)]
    pub enum Commands {
        /// Create Protobuf specs from MavLink message definitions  
        #[cfg(feature = "proto")]
        Proto {},
        #[cfg(feature = "rust")]
        Rust {
            /// Path to module.
            #[arg(short, long, default_value = DEFAULT_RUST_MODULE_PATH)]
            module_path: String,
            /// Add serde support.
            #[arg(short, long, default_value_t = false)]
            serde: bool,
        },
    }
}

#[cfg(any(feature = "proto", feature = "rust"))]
mod specs {
    use anyhow::anyhow;
    use mavspec::parser::XMLInspector;
    use mavspec::protocol::Protocol;

    pub fn parse_definitions(src: &[String]) -> anyhow::Result<Protocol> {
        XMLInspector::new(src.to_vec())?
            .parse()
            .map_err(|err| anyhow!(err))
    }
}

#[cfg(any(feature = "proto", feature = "rust"))]
mod process {
    use crate::cli::{Cli, Commands};
    use mavcodegen::rust::RustGeneratorParams;

    pub fn process(cli: &Cli) -> anyhow::Result<()> {
        match &cli.command {
            None => Ok(()),
            Some(command) => {
                let protocol = crate::specs::parse_definitions(&cli.src)?;
                let path = std::path::Path::new(&cli.out).to_path_buf();

                match command {
                    #[cfg(feature = "proto")]
                    Commands::Proto {} => {
                        mavcodegen::proto::ProtobufGenerator::new(protocol, path).generate()
                    }
                    #[cfg(feature = "rust")]
                    Commands::Rust { module_path, serde } => mavcodegen::rust::RustGenerator::new(
                        protocol,
                        path,
                        RustGeneratorParams {
                            module_path: module_path.clone(),
                            serde: *serde,
                        },
                    )
                    .generate(),
                }
            }
        }
    }
}

fn main() {
    use clap::Parser;

    // Setup logger
    env_logger::builder()
        // Suppress everything below `info` for third-party modules.
        .filter_level(log::LevelFilter::Info)
        // Allow everything from current package
        .filter_module(env!("CARGO_PKG_NAME"), log::LevelFilter::Trace)
        .init();

    // Parse CLI arguments:
    let cli = cli::Cli::parse();
    log::debug!("CLI arguments: {cli:?}");

    // Process CLI commands
    #[cfg(any(feature = "proto", feature = "rust"))]
    process::process(&cli).unwrap();
}
