use crate::normalise::NormalisedRecord;
use crate::publish::{PublishError, Publisher};

/// A custom parameterised-replaceable event kind (NIP-33's 30000-39999
/// range). Not registered anywhere; it only needs to stay consistent
/// across every event this project publishes, so a re-publish at the same
/// `d` tag actually replaces the prior one for subscribers.
pub const EVENT_KIND: u16 = 30_818;

/// Publishes normalised records as Nostr parameterised-replaceable events:
/// re-publishing at the same `d` tag (see `event_address`) replaces the
/// previous event for every subscriber, which is what makes an OParl
/// `modified` update or `deleted` marker a plain republish rather than a
/// separate correction mechanism. Optional: CCF's git-versioned `data/`
/// store remains the source of truth regardless of whether this is
/// configured. See `from_env`.
pub struct NostrPublisher {
    secret_key: String,
    relays: Vec<String>,
}

impl NostrPublisher {
    /// Reads CCF_NOSTR_SECRET_KEY and CCF_NOSTR_RELAYS (comma-separated).
    /// Returns None when no secret key is configured; the pull loop treats
    /// that as "skip publishing", not an error, since this is an additive
    /// broadcast layer, not a dependency the core feed needs to work.
    pub fn from_env() -> Option<Self> {
        let secret_key = std::env::var("CCF_NOSTR_SECRET_KEY").ok()?;
        let relays = std::env::var("CCF_NOSTR_RELAYS")
            .map(|value| value.split(',').map(str::trim).map(str::to_string).collect())
            .unwrap_or_default();
        Some(Self { secret_key, relays })
    }

    /// The event's `d` tag: what makes a later publish replace this one
    /// rather than create a new event. Deliberately keyed by council_id, not
    /// jurisdiction; see the doc comment on NormalisedRecord for why
    /// jurisdiction isn't tracked per-record yet.
    fn event_address(record: &NormalisedRecord) -> String {
        format!("{}:{:?}:{}", record.council_id, record.record_type, record.source_id)
    }
}

impl Publisher for NostrPublisher {
    fn publish(&self, record: &NormalisedRecord) -> Result<(), PublishError> {
        let _ = (&self.secret_key, &self.relays, Self::event_address(record));
        // Not implemented yet: build the NIP-33 event (kind EVENT_KIND, tags
        // ["d", event_address], ["council", record.council_id], ["type",
        // record.record_type]), sign it with secret_key, and send it to
        // each relay in `relays`. Needs a real Nostr client library choice
        // and at least one relay to test against before this is worth
        // writing for real.
        Err(PublishError::NotImplemented)
    }
}
