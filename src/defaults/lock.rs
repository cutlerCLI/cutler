use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::util::logging::{LogLevel, print_log};

lazy_static! {
    /// Map from domain - a mutex that serializes all write/delete calls
    static ref DOMAIN_LOCKS: Mutex<HashMap<String, Arc<Mutex<()>>>> = Mutex::new(HashMap::new());
}

/// Get (and create if missing) the lock for a given domain.
/// Cloning the Arc cheaply hands out the same mutex to all callers.
pub fn lock_for(domain: &str, verbose: bool) -> Arc<Mutex<()>> {
    if verbose {
        print_log(LogLevel::Info, &format!("Locking domain: {}", domain));
    }
    let mut map = DOMAIN_LOCKS.lock().unwrap();
    map.entry(domain.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}
