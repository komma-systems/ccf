# United Kingdom: adapter groundwork, not yet a survey

## Status

Zero verified endpoints. This directory exists to hold the UK's registry
and research once that survey starts; today it holds neither, on purpose.

## Why there's an adapter but no registry entries

Germany's `SourceAdapter` (OParl) came after a real endpoint survey (see
`jurisdictions/de/RESEARCH.md`): find candidates, verify them, then write
the adapter against real responses. The UK adapter (`ModernGovAdapter`, in
`src/source/moderngov.rs`) was written the other way round, deliberately:
to prove the `SourceAdapter` trait actually holds for a protocol that isn't
OParl, before spending time on a real UK survey. It normalises an embedded,
clearly-synthetic example payload - see `examples/moderngov_demo.rs` - not
a verified endpoint's response.

Do not read the ModernGov adapter's field mapping as a claim about
ModernGov's real API shape. It hasn't been checked against a live endpoint.

## Scale

- 307 local authorities in England (May 2026, falling as county/district
  areas convert to unitary authorities under ongoing local government
  reorganisation).
- 32 unitary authorities in Scotland, 22 in Wales, 11 in Northern Ireland.
- About 372 principal authorities in total, plus a separate lower tier of
  roughly 10,000 town, parish, and community councils, out of scope for a
  first adapter.

## Why this is harder than Germany's survey

No UK-wide standard equivalent to OParl exists. Councils publish
committee/agenda data through several proprietary systems - ModernGov,
CMIS, Egenda, and others - each with its own API or none at all. Unlike
`jurisdictions/de/RESEARCH.md`, which started from a single
community-maintained endpoint list (`OParl/resources`), there is no
equivalent single seed list here yet: the first real step is finding out
which vendor each of the ~372 authorities actually runs, before any
endpoint can even be attempted.

## Next steps

1. Survey which committee-management vendor each principal authority
   actually runs (no existing single list to start from; this is the part
   that hasn't begun).
2. Pick one real, responsive endpoint and verify it by hand, the same way
   Germany's four were verified: a single `GET`, checked manually.
3. Rewrite `ModernGovAdapter::pull` to fetch that real endpoint and map its
   actual field names, replacing the embedded example payload.
4. Add the verified entry to `registry.yaml` with `status: verified`, and
   update the main README's Coverage table once it's real.
