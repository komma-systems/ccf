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
/// Organizations, people, agenda items, and files are not walked yet — see
/// the main README's "next steps".
pub struct OParlAdapter;

impl SourceAdapter for OParlAdapter {
    fn pull(&self, council_id: &str, endpoint_url: &str) -> Result<Vec<NormalisedRecord>, NormaliseError> {
        let system = get_json(endpoint_url)?;
        let body_list_url = require_str(&system, "body", endpoint_url)?;

        let mut records = Vec::new();
        for body in paginate(&body_list_url)? {
            let meeting_url = body.get("meeting").and_then(Value::as_str).map(str::to_string);
            let paper_url = body.get("paper").and_then(Value::as_str).map(str::to_string);

            if let Some(meeting_url) = meeting_url {
                for meeting in paginate(&meeting_url)? {
                    records.push(to_record(council_id, RecordType::Meeting, meeting)?);
                }
            }

            if let Some(paper_url) = paper_url {
                for paper in paginate(&paper_url)? {
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
        .call()
        .map_err(|error| NormaliseError::Request { url: url.to_string(), source: Box::new(error) })?;
    response
        .into_json()
        .map_err(|error| NormaliseError::InvalidJson { url: url.to_string(), source: error })
}

/// Follows `links.next` until it's absent, repeated, or a page comes back
/// empty. A page count cap guards against a server that loops `next` forever
/// instead of terminating.
fn paginate(first_url: &str) -> Result<Vec<Value>, NormaliseError> {
    const MAX_PAGES: usize = 500;

    let mut items = Vec::new();
    let mut next_url = Some(first_url.to_string());
    let mut previous_url: Option<String> = None;

    for _ in 0..MAX_PAGES {
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
