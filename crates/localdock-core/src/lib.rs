pub mod error;
pub mod loopback_env;
pub mod path_guard;
pub mod ports;
pub mod process_tree;
pub mod registry;
pub mod scanner;
pub mod spawn;
pub mod version;

pub use error::LocalDockError;
pub use scanner::{scan_root, ProposedApp};

pub fn crate_name() -> &'static str {
    "localdock-core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_is_localdock_core() {
        assert_eq!(crate_name(), "localdock-core");
    }
}
