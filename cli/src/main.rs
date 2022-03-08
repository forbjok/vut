use clap::Parser;
use log::{debug, LevelFilter};

mod command;
mod error;
mod ui;

use vut::project::BumpVersion;

#[derive(Debug, Parser)]
#[clap(name = "Vut", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opt {
    #[clap(short = 'v', parse(from_occurrences), help = "Verbosity")]
    verbosity: u8,
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
        #[structopt(name = "format", help = "Output format (json)")]
        format: String,
    },

    #[clap(name = "set", about = "Set version")]
    Set {
        #[structopt(name = "version", help = "Version to set")]
        version: String,
    },

    #[clap(name = "bump", about = "Bump version")]
    Bump {
        #[structopt(help = "Version to bump (major|minor|prerelease|build)")]
        bump_version: BumpVersion,
    },

    #[clap(name = "generate", alias = "gen", about = "Generate template output")]
    Generate,
}

fn main() {
    let opt = Opt::parse();

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    let log_level = match opt.verbosity {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    // Initialize logging
    initialize_logging(log_level);

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

fn initialize_logging(our_level_filter: LevelFilter) {
    use chrono::Utc;

    fern::Dispatch::new()
        .level(our_level_filter)
        .chain(std::io::stderr())
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} | {} | {} | {}",
                Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.target(),
                record.level(),
                message
            ))
        })
        .apply()
        .unwrap();
}
