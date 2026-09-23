mod normalise;
mod publish;
mod registry;
mod source;

use std::path::Path;

use publish::nostr::NostrPublisher;
use publish::Publisher;
use registry::SourceFormat;
use source::oparl::OParlAdapter;
use source::SourceAdapter;

/// For each verified registry entry, pull its source output and normalise
/// it. Writing the result under data/<jurisdiction>/<council-id>/ and
/// committing it is not implemented yet — see the main README's "next
/// steps". This binary exists to fix the shape: discover -> pull ->
/// normalise -> version -> commit.
fn main() {
    // Only Germany exists as a jurisdiction so far; a second one gets its own
    // registry path alongside this rather than a CLI flag, until there's a
    // second one to actually branch on.
    let registry_path = Path::new("jurisdictions/de/registry.yaml");
    let registry = match registry::Registry::load(registry_path) {
        Ok(registry) => registry,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    println!("Loaded {} council(s):", registry.councils.len());
    for council in &registry.councils {
        println!("  {} — {} ({})", council.id, council.name, council.endpoint_url);
    }

    // Absence means "skip publishing" (see NostrPublisher::from_env) — the
    // git feed works standalone regardless. Presence just means someone
    // will see "not implemented" errors below rather than nothing.
    let nostr_publisher = NostrPublisher::from_env();
    println!(
        "\nNostr publish: {}",
        if nostr_publisher.is_some() { "configured" } else { "not configured, skipping" }
    );

    println!("\nPulling verified councils:");
    let mut had_error = false;
    for council in registry.councils.iter().filter(|c| c.status == registry::CouncilStatus::Verified) {
        let result = match council.source_format {
            SourceFormat::OParl => OParlAdapter.pull(&council.id, &council.endpoint_url),
        };

        match result {
            Ok(records) => {
                println!("  {} — {} record(s) pulled", council.id, records.len());

                if let Some(publisher) = &nostr_publisher {
                    // Best-effort and non-blocking, on purpose: git stays
                    // the source of truth, so one relay/record failing here
                    // must never fail the pull.
                    for record in &records {
                        if let Err(error) = publisher.publish(record) {
                            eprintln!("  {} — nostr publish failed for {}: {error}", council.id, record.source_id);
                        }
                    }
                }
            }
            Err(error) => {
                had_error = true;
                eprintln!("  {} — pull failed: {error}", council.id);
            }
        }
    }

    println!("\nWriting to data/ and committing is not implemented yet.");
    if had_error {
        std::process::exit(1);
    }
}
