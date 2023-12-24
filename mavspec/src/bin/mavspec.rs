mod cli {
    use clap::{Parser, Subcommand};

    const DEFAULT_OUT_PATH: &str = ".";
    const DELIMITER: char = ' ';

    /// MavLink code generator.
    #[derive(Debug, Parser)]
    #[command(author, version, about, long_about = None)]
    pub struct Cli {
        #[command(subcommand)]
        pub command: Option<Commands>,

        /// List of sources.
        #[arg(short = 's', long, value_parser, num_args = 1.., value_delimiter = DELIMITER)]
        pub src: Vec<String>,

        /// Output path.
        #[arg(short = 'o', long, default_value = DEFAULT_OUT_PATH)]
        pub out: String,

        /// Clean output path. Otherwise files will be added incrementally.
        #[arg(short = 'c', long, default_value_t = false)]
        pub clean: bool,
    }

    /// Cli commands
    #[derive(Debug, Subcommand)]
    pub enum Commands {
        /// Generate Rust bindings from MavLink message definitions
        #[cfg(feature = "rust")]
        Rust {
            /// Add serde support.
            #[arg(short = 's', long, default_value_t = false)]
            serde: bool,
            /// Messages to generate.
            ///
            /// By default only enums required for these messages will be generated. If you want all enums, use
            /// `--all-enums` flag.
            #[arg(short = 'M', long, value_parser, num_args = 1.., value_delimiter = DELIMITER)]
            messages: Option<Vec<String>>,
            /// Include all enums regardless of specified messages.
            #[arg(short = 'a', long, default_value_t = false)]
            all_enums: bool,
        },
    }
}

#[cfg(feature = "rust_gen")]
mod process {
    use crate::cli::{Cli, Commands};
    use std::fs::remove_dir_all;

    pub fn process(cli: &Cli) -> anyhow::Result<()> {
        match &cli.command {
            None => Ok(()),
            Some(command) => {
                log::info!("MAVLink XML definition sources dirs: {:?}", cli.src);

                let out_path = std::path::Path::new(&cli.out).to_path_buf();

                if cli.clean {
                    log::warn!("Output path will be cleaned.");
                    if let Err(err) = remove_dir_all(&out_path) {
                        log::debug!("Error cleaning directory: {err:?}.");
                    };
                }

                match command {
                    #[cfg(feature = "rust")]
                    Commands::Rust {
                        serde,
                        messages,
                        all_enums,
                    } => {
                        log::info!("Writing Rust bindings to output path: {:?}", out_path);

                        let mut conf = mavspec::rust::gen::BuildHelper::builder(&out_path);

                        if let Some(messages) = messages {
                            conf.set_messages(messages);
                        };
                        let sources: Vec<&str> = cli.src.iter().map(|s| s.as_str()).collect();

                        conf.set_sources(&sources)
                            .set_all_enums(*all_enums)
                            .set_serde(*serde)
                            .generate()
                    }
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
    #[cfg(feature = "rust_gen")]
    process::process(&cli).unwrap();
}
