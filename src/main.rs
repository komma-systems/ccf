mod normalise;
mod publish;
mod registry;
mod source;
mod store;

use std::path::Path;

use normalise::NormalisedRecord;
use publish::nostr::NostrPublisher;
use publish::Publisher;
use registry::SourceFormat;
use source::oparl::OParlAdapter;
use source::SourceAdapter;

/// For each verified registry entry, pull its source output, normalise it,
/// write it under data/<jurisdiction>/<council-id>/, and commit the result
/// as one commit per run. This binary is the shape the whole project is
/// built around: discover, pull, normalise, version, commit.
fn main() {
    // Only Germany exists as a jurisdiction so far; a second one gets its own
    // registry path alongside this rather than a CLI flag, until there's a
    // second one to actually branch on.
    let jurisdiction = "de";
    let repo_root = Path::new(".");
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
        println!("  {}: {} ({})", council.id, council.name, council.endpoint_url);
    }

    // Absence means "skip publishing" (see NostrPublisher::from_env); the
    // git feed works standalone regardless. Presence just means someone
    // will see "not implemented" errors below rather than nothing.
    let nostr_publisher = NostrPublisher::from_env();
    println!(
        "\nNostr publish: {}",
        if nostr_publisher.is_some() { "configured" } else { "not configured, skipping" }
    );

    println!("\nPulling verified councils:");
    let mut had_error = false;
    let mut all_records: Vec<NormalisedRecord> = Vec::new();
    let mut summary_lines: Vec<String> = Vec::new();

    for council in registry.councils.iter().filter(|c| c.status == registry::CouncilStatus::Verified) {
        let result = match council.source_format {
            SourceFormat::OParl => OParlAdapter.pull(&council.id, &council.endpoint_url),
        };

        match result {
            Ok(records) => {
                println!("  {}: {} record(s) pulled", council.id, records.len());
                summary_lines.push(format!("- {}: {} record(s)", council.id, records.len()));

                if let Some(publisher) = &nostr_publisher {
                    // Best-effort and non-blocking, on purpose: git stays
                    // the source of truth, so one relay/record failing here
                    // must never fail the pull.
                    for record in &records {
                        if let Err(error) = publisher.publish(record) {
                            eprintln!("  {}: nostr publish failed for {}: {error}", council.id, record.source_id);
                        }
                    }
                }

                all_records.extend(records);
            }
            Err(error) => {
                had_error = true;
                eprintln!("  {}: pull failed: {error}", council.id);
            }
        }
    }

    println!("\nWriting {} record(s) to data/:", all_records.len());
    let write_result = std::fs::create_dir_all(repo_root.join("data"))
        .map_err(|source| store::StoreError::Io { path: repo_root.join("data"), source })
        .and_then(|_| store::write_records(&repo_root.join("data"), jurisdiction, &all_records));

    match write_result {
        Ok(written) => {
            println!("  wrote {written} file(s)");

            let message = format!("Pull: {} council(s), {} record(s)\n\n{}", summary_lines.len(), all_records.len(), summary_lines.join("\n"));
            match store::commit(repo_root, &message) {
                Ok(Some(hash)) => println!("  committed as {hash}"),
                Ok(None) => println!("  nothing changed, no commit made"),
                Err(error) => {
                    had_error = true;
                    eprintln!("  commit failed: {error}");
                }
            }
        }
        Err(error) => {
            had_error = true;
            eprintln!("  write failed: {error}");
        }
    }

    if had_error {
        std::process::exit(1);
    }
}
