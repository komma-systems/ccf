use serde_json::Value;

use crate::normalise::{NormaliseError, NormalisedRecord, RecordType};
use crate::source::SourceAdapter;

/// A second SourceAdapter for a protocol that isn't OParl at all, to prove
/// the trait actually holds rather than being designed around OParl and
/// never tested against anything else. ModernGov is a real UK committee-
/// management vendor (see jurisdictions/uk/RESEARCH.md), but no endpoint has
/// been verified yet, so `pull` doesn't make a network request: it
/// normalises an embedded, clearly-synthetic example payload instead. The
/// point of this adapter today is the shape of `pull`, not a coverage
/// claim; jurisdictions/uk/registry.yaml has zero verified councils, and
/// the main README's Coverage table stays honest about that.
pub struct ModernGovAdapter;

impl SourceAdapter for ModernGovAdapter {
    fn pull(&self, council_id: &str, _endpoint_url: &str) -> Result<Vec<NormalisedRecord>, NormaliseError> {
        // A real implementation would `ureq::get(endpoint_url)` here, the
        // same as OParlAdapter does. There is nothing to GET yet: this
        // fixture exists so the adapter's mapping logic - deliberately
        // using different field names than OParl's (committeeId, not id;
        // lastModified, not modified; isDeleted, not deleted) - can be
        // exercised end to end. See examples/moderngov_demo.rs to run it.
        let payload: Value = serde_json::from_str(EXAMPLE_PAYLOAD).map_err(|source| NormaliseError::InvalidJson {
            url: "embedded example payload, not a real endpoint".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, source),
        })?;

        let mut records = Vec::new();
        records.extend(map_array(&payload, "committees", "committeeId", "lastModified", "isDeleted", RecordType::Organization, council_id)?);
        records.extend(map_array(&payload, "meetings", "meetingId", "lastModified", "isDeleted", RecordType::Meeting, council_id)?);
        records.extend(map_array(&payload, "documents", "documentId", "lastModified", "isDeleted", RecordType::Paper, council_id)?);
        Ok(records)
    }
}

fn map_array(
    payload: &Value,
    array_field: &str,
    id_field: &str,
    modified_field: &str,
    deleted_field: &str,
    record_type: RecordType,
    council_id: &str,
) -> Result<Vec<NormalisedRecord>, NormaliseError> {
    let array = payload
        .get(array_field)
        .and_then(Value::as_array)
        .ok_or_else(|| NormaliseError::UnexpectedShape {
            url: "embedded example payload".to_string(),
            reason: format!("missing or non-array \"{array_field}\""),
        })?;

    array
        .iter()
        .map(|item| {
            let source_id = item
                .get(id_field)
                .and_then(Value::as_str)
                .ok_or_else(|| NormaliseError::UnexpectedShape {
                    url: "embedded example payload".to_string(),
                    reason: format!("missing or non-string \"{id_field}\""),
                })?
                .to_string();
            let modified_at = item.get(modified_field).and_then(Value::as_str).unwrap_or_default().to_string();
            let deleted = item.get(deleted_field).and_then(Value::as_bool).unwrap_or(false);

            Ok(NormalisedRecord {
                council_id: council_id.to_string(),
                record_type,
                source_id,
                deleted_at: deleted.then(|| modified_at.clone()),
                modified_at,
                data: item.clone(),
            })
        })
        .collect()
}

/// Illustrative only: a plausible committee-management shape, not a
/// verified ModernGov response. Field names deliberately differ from
/// OParl's so mapping them is a real exercise of the adapter, not a
/// coincidence of reusing the same names.
const EXAMPLE_PAYLOAD: &str = r#"{
  "committees": [
    { "committeeId": "cmt-planning", "name": "Planning Committee", "lastModified": "2026-09-02T09:00:00+01:00", "isDeleted": false }
  ],
  "meetings": [
    { "meetingId": "mtg-2026-10-12-planning", "committeeId": "cmt-planning", "title": "Planning Committee", "date": "2026-10-12", "lastModified": "2026-09-18T14:30:00+01:00", "isDeleted": false }
  ],
  "documents": [
    { "documentId": "doc-2026-10-12-agenda", "meetingId": "mtg-2026-10-12-planning", "title": "Agenda", "lastModified": "2026-09-18T14:30:00+01:00", "isDeleted": false },
    { "documentId": "doc-2026-09-05-minutes-superseded", "meetingId": "mtg-2026-09-05-planning", "title": "Minutes (superseded by correction)", "lastModified": "2026-09-20T11:05:00+01:00", "isDeleted": true }
  ]
}"#;
