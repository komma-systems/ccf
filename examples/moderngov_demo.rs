//! Demonstrates the second SourceAdapter end to end: `cargo run --example
//! moderngov_demo`. This does not touch the real pull loop or the German
//! registry in `src/main.rs` - jurisdictions/uk/registry.yaml has zero
//! verified councils, so nothing here claims UK coverage. It exists to show
//! what happens when a council speaks something other than OParl: a
//! different adapter, mapping different field names, into the exact same
//! NormalisedRecord shape.

use ccf::source::moderngov::ModernGovAdapter;
use ccf::source::SourceAdapter;

fn main() {
    let records = match ModernGovAdapter.pull("sample-uk-council", "https://example.invalid/moderngov") {
        Ok(records) => records,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    println!("ModernGovAdapter pulled {} record(s), same shape as OParl's:\n", records.len());
    for record in &records {
        println!(
            "  {:?} {} (modified {}{})",
            record.record_type,
            record.source_id,
            record.modified_at,
            if record.deleted_at.is_some() { ", deleted" } else { "" }
        );
    }
}
