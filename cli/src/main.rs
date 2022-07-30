use clap::Parser;

mod command;
mod error;
mod ui;

use tracing::debug;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use vut::project::BumpVersion;

#[derive(Debug, Parser)]
#[clap(name = "Vut", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opt {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    #[clap(name = "init", about = "Initialize version file")]
    Init {
        #[clap(long = "example", help = "Create example configuration")]
        example: bool,

        #[clap(
            short = 'f',
            long = "force",
            help = "Proceed even if inside the scope of an existing Vut configuration"
        )]
        force: bool,

        #[clap(name = "version", help = "Specify initial version")]
        version: Option<String>,
    },

    #[clap(name = "get", about = "Get version")]
    Get {
        #[clap(name = "format", help = "Output format (json)")]
        format: String,
    },

    #[clap(name = "set", about = "Set version")]
    Set {
        #[clap(name = "version", help = "Version to set")]
        version: String,
    },

    #[clap(name = "bump", about = "Bump version")]
    Bump {
        #[clap(help = "Version to bump (major|minor|prerelease|build)")]
        bump_version: BumpVersion,
    },

    #[clap(name = "generate", alias = "gen", about = "Generate template output")]
    Generate,
}

fn main() {
    let opt = Opt::parse();

    // Initialize logging
    initialize_logging();

    debug!("Debug logging enabled.");

    let cmd_result = match opt.command {
        Command::Bump { bump_version } => command::bump(bump_version),
        Command::Generate => command::generate(),
        Command::Get { format } => command::get(&format),
        Command::Init {
            example,
            force,
            version,
        } => command::init(example, force, version.as_deref()),
        Command::Set { version } => command::set(&version),
    };

    match cmd_result {
        Ok(_) => {}
        Err(err) => {
            // Print error description to stderr
            eprintln!("{}", err.description);

            // Return the exit code that corresponds to the error kind
            std::process::exit(err.kind.exit_code());
        }
    };
}

fn initialize_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")))
        .with_writer(std::io::stderr)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default tracing subscriber failed!");
}
