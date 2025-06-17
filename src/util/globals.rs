use std::sync::atomic::{AtomicBool, Ordering};

/*
 * These are primarily used by functions / functionality which are out of the typical commands scheme.
 *
 * They often serve as a replica for their global argument counterparts,
 * "just in case".
 */

// --accept-interactive
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

// --verbose
static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(value: bool) {
    VERBOSE.store(value, Ordering::SeqCst);
}

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::SeqCst)
}

// --dry-run
static DRY_RUN: AtomicBool = AtomicBool::new(false);

pub fn set_dry_run(value: bool) {
    DRY_RUN.store(value, Ordering::SeqCst);
}

pub fn should_dry_run() -> bool {
    DRY_RUN.load(Ordering::SeqCst)
}

// --no-restart-services
static NO_RESTART_SERVICES: AtomicBool = AtomicBool::new(false);

pub fn set_no_restart_services(value: bool) {
    NO_RESTART_SERVICES.store(value, Ordering::SeqCst);
}

pub fn should_not_restart_services() -> bool {
    NO_RESTART_SERVICES.load(Ordering::SeqCst)
}
