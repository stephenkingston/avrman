use avrman::error::AvrResult;
use clap::{Parser, command};
use program::{ProgramOptions, handle_programming};

mod program;

#[derive(Parser, Debug, Clone)]
#[command(version, long_about = None)]
enum Cli {
    /// Program target device with options
    #[command(name = "program", alias = "p")]
    Program(ProgramOptions),
}

fn main() -> AvrResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli = Cli::parse();

    match cli {
        Cli::Program(opts) => handle_programming(opts)?,
    }

    Ok(())
}
