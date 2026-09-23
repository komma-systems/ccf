pub mod nostr;

use thiserror::Error;

use crate::normalise::NormalisedRecord;

/// A sink CCF can mirror normalised records to, beyond the git-versioned
/// `data/` store that remains the source of truth. Nostr is the first
/// implementation; a webhook or push-MCP sink would get its own module
/// behind this same trait rather than a special case in the pull loop.
pub trait Publisher {
    fn publish(&self, record: &NormalisedRecord) -> Result<(), PublishError>;
}

#[derive(Debug, Error)]
pub enum PublishError {
    #[error("not implemented: no relay connection or event-signing has been wired up yet")]
    NotImplemented,
}
