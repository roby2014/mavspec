mod cli {
    use clap::{Parser, Subcommand};

    const DEFAULT_OUT_PATH: &str = ".";
    const DELIMITER: char = ' ';

    /// MAVLink code generator.
    #[derive(Debug, Parser)]
    #[command(author, version, about, long_about = None)]
    #[command(arg_required_else_help = true)]
    pub struct Cli {
        #[command(subcommand)]
        pub command: Option<Commands>,

        /// List of sources.
        #[arg(short = 's', long, value_parser, num_args = 1.., value_delimiter = DELIMITER)]
        pub src: Vec<String>,

        /// Output path.
        #[arg(short = 'o', long, default_value = DEFAULT_OUT_PATH)]
        pub out: String,

        /// Clean output path. Otherwise, files will be added incrementally.
        #[arg(short = 'c', long, default_value_t = false)]
        pub clean: bool,
    }

    /// Cli commands
    #[derive(Debug, Subcommand)]
    pub enum Commands {
        /// Generate Rust bindings from MAVLink message definitions
        #[cfg(feature = "rust")]
        Rust {
            /// Enable Serde support.
            #[arg(short = 's', long, default_value_t = false)]
            serde: bool,
            /// Microservices to generate.
            ///
            /// This option will filter out irrelevant messages, enums, and enum entries.
            ///
            /// It is possible to add additional enums, messages, and commands via `--enums`, `--messages`, and
            /// `--commands` respectively.
            #[arg(short = 'M', long, value_parser, num_args = 1.., value_delimiter = DELIMITER)]
            microservices: Option<Vec<String>>,
            /// Messages to generate.
            ///
            /// These commands will be added to whatever commands are required by microservices specified by
            /// `--microservices`.
            ///
            /// This option will filter out irrelevant enums.
            ///
            /// Postfix wildcards are accepted (i.e. `PREFIX_*`).
            #[arg(short = 'm', long, value_parser, num_args = 1.., value_delimiter = DELIMITER)]
            messages: Option<Vec<String>>,
            /// Enums to generate.
            ///
            /// Postfix wildcards are accepted (i.e. `PREFIX_*`).
            #[arg(short = 'e', long, value_parser, num_args = 1.., value_delimiter = DELIMITER)]
            enums: Option<Vec<String>>,
            /// Commands to generate.
            ///
            /// These commands will be added to whatever commands are required by microservices specified by
            /// `--microservices`.
            ///
            /// This option will filter out irrelevant enums and enum entries (particularly of `MAV_CMD` enum). If set,
            /// it will enable messages relevant for command protocol.
            ///
            /// Postfix wildcards are accepted (i.e. `PREFIX_*`).
            #[arg(short = 'c', long, value_parser, num_args = 1.., value_delimiter = DELIMITER)]
            commands: Option<Vec<String>>,
            /// Generate tests.
            #[arg(short = 't', long, default_value_t = false)]
            generate_tests: bool,
        },
    }
}

#[cfg(feature = "rust_gen")]
mod process {
    use crate::cli::{Cli, Commands};
    use mavspec::rust::gen::error::RustGenResult;
    use std::fs::remove_dir_all;

    pub fn process(cli: &Cli) -> RustGenResult<()> {
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
                        microservices,
                        messages,
                        enums,
                        commands,
                        generate_tests,
                    } => {
                        log::info!("Writing Rust bindings to output path: {:?}", out_path);

                        let mut builder = mavspec::rust::gen::BuildHelper::builder(&out_path);

                        if let Some(microservices) = microservices {
                            builder.set_microservices(microservices);
                        };
                        if let Some(messages) = messages {
                            builder.set_messages(messages);
                        };
                        if let Some(enums) = enums {
                            builder.set_enums(enums);
                        };
                        if let Some(commands) = commands {
                            builder.set_commands(commands);
                        };

                        let sources: Vec<&str> = cli.src.iter().map(|s| s.as_str()).collect();

                        builder
                            .set_sources(&sources)
                            .set_serde(*serde)
                            .set_generate_tests(*generate_tests)
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
    env_logger::init();

    // Parse CLI arguments:
    let cli = cli::Cli::parse();
    log::trace!("CLI arguments: {cli:?}");

    // Process CLI commands
    #[cfg(feature = "rust_gen")]
    process::process(&cli).unwrap();
}
