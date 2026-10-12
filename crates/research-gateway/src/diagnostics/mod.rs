mod attempt;
mod evidence;
mod sink;
mod timing;

pub use attempt::GatewayAttempt;
pub use sink::GatewayAttemptSink;

pub(crate) use evidence::{fingerprint, response_id_from_body};
pub(crate) use sink::NoopAttemptSink;
pub(crate) use timing::elapsed_millis;

#[cfg(test)]
mod tests;
