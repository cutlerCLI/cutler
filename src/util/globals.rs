use std::sync::atomic::{AtomicBool, Ordering};

/*
 * These are primarily used by functions / functionality which are out of the typical commands scheme.
 *
 * They often serve as a replica for their global argument counterparts,
 * "just in case".
 */

// --accept-all
static ACCEPT_INTERACTIVE: AtomicBool = AtomicBool::new(false);

pub fn set_accept_interactive(value: bool) {
    ACCEPT_INTERACTIVE.store(value, Ordering::SeqCst);
}

pub fn should_accept_interactive() -> bool {
    ACCEPT_INTERACTIVE.load(Ordering::SeqCst)
}

// --quiet
static QUIET: AtomicBool = AtomicBool::new(false);

pub fn set_quiet(value: bool) {
    QUIET.store(value, Ordering::SeqCst);
}

pub fn should_be_quiet() -> bool {
    QUIET.load(Ordering::SeqCst)
}
