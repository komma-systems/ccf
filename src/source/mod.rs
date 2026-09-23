pub mod oparl;

use crate::normalise::{NormaliseError, NormalisedRecord};

/// One jurisdiction's ingestion protocol. OParl is the first and only
/// implementation; a future jurisdiction that doesn't speak OParl gets its
/// own adapter behind this same trait rather than a special case in the
/// pull loop.
pub trait SourceAdapter {
    fn pull(&self, council_id: &str, endpoint_url: &str) -> Result<Vec<NormalisedRecord>, NormaliseError>;
}
