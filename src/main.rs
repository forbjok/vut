#![allow(dead_code)]

use log::{debug, LevelFilter};
use structopt::StructOpt;

mod command;
mod file_updater;
mod template;
mod util;
mod version;
mod version_source;
mod vut;

use vut::BumpVersion;

#[derive(StructOpt, Debug)]
#[structopt(name = "Vut", version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opt {
    #[structopt(short = "v", parse(from_occurrences), help = "Verbosity")]
    verbosity: u8,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "init", about = "Initialize version file")]
    Init {
        #[structopt(name = "version", about = "Specify initial version")]
        version: Option<String>,
    },

    #[structopt(name = "get", about = "Get version")]
    Get {
        #[structopt(name = "format", about = "Output format (json)")]
        format: String,
    },

    #[structopt(name = "set", about = "Set version")]
    Set {
        #[structopt(name = "version", about = "Version to set")]
        version: String,
    },

    #[structopt(name = "bump", about = "Bump version")]
    Bump {
        #[structopt(help = "Version to bump (major|minor|prerelease|build)")]
        bump_version: BumpVersion,
    },

    #[structopt(name = "generate", alias = "gen", about = "Generate template output")]
    Generate,
}

fn main() {
    let opt = Opt::from_args();

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    let log_level = match opt.verbosity {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 | _ => LevelFilter::Trace,
    };

    // Initialize logging
    initialize_logging(log_level);

    debug!("Debug logging enabled.");

    let cmd_result = match opt.command {
        Command::Bump { bump_version } => command::bump(bump_version),
        Command::Generate => command::generate(),
        Command::Get { format } => command::get(&format),
        Command::Init { version } => command::init(version.as_ref().map(|v| v.as_str())),
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
