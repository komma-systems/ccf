use serde_json::Value;

use crate::normalise::{NormaliseError, NormalisedRecord, RecordType};
use crate::source::SourceAdapter;

/// Walks `system -> body -> {meeting, paper}` and normalises each object
/// found along the way. Deliberately generic over the JSON shape rather than
/// typed OParl structs: vendors are known to vary in which optional fields
/// they populate, and the only fields this adapter actually depends on are
/// `id`, `modified`, `deleted`, and the handful of URL fields used to walk
/// the graph. See jurisdictions/de/RESEARCH.md for the vendors observed.
///
/// Organisations, people, agenda items, and files are not walked yet; see
/// the main README's "next steps".
pub struct OParlAdapter;

impl SourceAdapter for OParlAdapter {
    fn pull(&self, council_id: &str, endpoint_url: &str) -> Result<Vec<NormalisedRecord>, NormaliseError> {
        let system = get_json(endpoint_url)?;
        let body_list_url = require_str(&system, "body", endpoint_url)?;

        // Meeting/paper history goes back years at ~25 items per page (see
        // RESEARCH.md); OParl's base spec has no recency filter to ask the
        // server for only recent items, so for now this caps *pages*
        // fetched rather than pulling full history. A real recency cutoff
        // (stop once `modified` predates some watermark) replaces this once
        // the write/commit step exists to make incremental pulls worthwhile
        // - see the main README's "next steps".
        const LIST_PAGE_CAP: usize = 2;

        let mut records = Vec::new();
        for body in paginate(&body_list_url, LIST_PAGE_CAP)? {
            let meeting_url = body.get("meeting").and_then(Value::as_str).map(str::to_string);
            let paper_url = body.get("paper").and_then(Value::as_str).map(str::to_string);

            if let Some(meeting_url) = meeting_url {
                for meeting in paginate(&meeting_url, LIST_PAGE_CAP)? {
                    records.push(to_record(council_id, RecordType::Meeting, meeting)?);
                }
            }

            if let Some(paper_url) = paper_url {
                for paper in paginate(&paper_url, LIST_PAGE_CAP)? {
                    records.push(to_record(council_id, RecordType::Paper, paper)?);
                }
            }

            records.push(to_record(council_id, RecordType::Body, body)?);
        }

        Ok(records)
    }
}

fn get_json(url: &str) -> Result<Value, NormaliseError> {
    let response = ureq::get(url)
        .timeout(std::time::Duration::from_secs(15))
        .call()
        .map_err(|error| NormaliseError::Request { url: url.to_string(), source: Box::new(error) })?;
    response
        .into_json()
        .map_err(|error| NormaliseError::InvalidJson { url: url.to_string(), source: error })
}

/// Follows `links.next` until it's absent, repeated, or `max_pages` is
/// reached. The cap is a hard stop, not just a runaway guard; see the
/// LIST_PAGE_CAP comment in `pull` for why it's small today.
fn paginate(first_url: &str, max_pages: usize) -> Result<Vec<Value>, NormaliseError> {
    let mut items = Vec::new();
    let mut next_url = Some(first_url.to_string());
    let mut previous_url: Option<String> = None;

    for _ in 0..max_pages {
        let Some(url) = next_url.take() else { break };
        if previous_url.as_deref() == Some(url.as_str()) {
            break;
        }

        let page = get_json(&url)?;
        let data = page
            .get("data")
            .and_then(Value::as_array)
            .ok_or_else(|| NormaliseError::UnexpectedShape {
                url: url.clone(),
                reason: "missing or non-array \"data\"".to_string(),
            })?;
        items.extend(data.iter().cloned());

        next_url = page
            .get("links")
            .and_then(|links| links.get("next"))
            .and_then(Value::as_str)
            .map(str::to_string);
        previous_url = Some(url);
    }

    Ok(items)
}

fn to_record(council_id: &str, record_type: RecordType, object: Value) -> Result<NormalisedRecord, NormaliseError> {
    let source_id = require_str(&object, "id", "<paginated object>")?;
    let modified_at = object
        .get("modified")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let deleted = object.get("deleted").and_then(Value::as_bool).unwrap_or(false);

    Ok(NormalisedRecord {
        council_id: council_id.to_string(),
        record_type,
        source_id,
        deleted_at: deleted.then(|| modified_at.clone()),
        modified_at,
        data: object,
    })
}

fn require_str(value: &Value, field: &str, context_url: &str) -> Result<String, NormaliseError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| NormaliseError::UnexpectedShape {
            url: context_url.to_string(),
            reason: format!("missing or non-string \"{field}\""),
        })
}
