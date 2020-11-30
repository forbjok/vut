use vut::project::VutCallbacks;

mod bump;
mod generate;
mod get;
mod init;
mod set;

pub use bump::*;
pub use generate::*;
pub use get::*;
pub use init::*;
pub use set::*;

fn stderr_vut_callbacks() -> VutCallbacks {
    VutCallbacks {
        deprecated: Some(Box::new(|m| eprintln!("DEPRECATED: {}", m))),
    }
}
