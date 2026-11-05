mod application;
mod command_registry;
mod error;
mod state;

pub(crate) use state::AppState;

pub(crate) fn run() {
    application::run();
}

#[cfg(all(test, windows, football_native_log_acceptance))]
mod native_log_acceptance;
