pub mod collector;
pub use collector::{
    check_exists, // Result<(),_>
    collect,      // HashMap<String,Table>
    effective,    // (domain,key)
    needs_prefix, // bool
    read_current, // Option<String>
};
