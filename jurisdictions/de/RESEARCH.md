# Germany: OParl endpoint survey

## Scale

Germany has roughly 10,780 independent municipalities (Gemeinden, Destatis
count, mid-2020s) plus around 400 district-level bodies (Landkreise and
kreisfreie Städte). The number of distinct RIS/OParl *endpoints* is lower
than that: many small Gemeinden don't run their own system at all, and are
served through a shared Amt or Verwaltungsgemeinschaft endpoint covering
several villages at once. No full count of live endpoints exists yet — that
is what this survey starts to establish, at a scale of a handful of
endpoints, not thousands.

## Sources used to find candidates

- [OParl/resources `endpoints.yml`](https://github.com/OParl/resources/blob/main/endpoints.yml) —
  the community-maintained canonical list.
- [dev.oparl.org/api/endpoints](https://dev.oparl.org/api/endpoints) — a live
  registry with last-checked status, useful for cross-checking staleness.

Both are lists of *claimed* endpoints, not verified ones — about half of what
was sampled here turned out dead, blocked, or moved.

## Method

One unauthenticated `GET` per candidate `system` URL, no custom headers, no
session/cookie handling. This means some failures below are the survey
method's limitation, not necessarily the endpoint's: a bot-check page or a
403 doesn't prove the endpoint is gone, only that it isn't reachable by a
plain GET.

## Verified live (in `registry.yaml`)

| Council | Endpoint | OParl version | Vendor |
|---|---|---|---|
| Stadt Köln | `buergerinfo.stadt-koeln.de/oparl/system` | 1.1 | SOMACOS |
| Stadt Münster | `oparl.stadt-muenster.de/system` | 1.1 | SOMACOS |
| Stadt Wuppertal | `oparl.wuppertal.de/oparl/system` | 1.1 | SOMACOS |
| Stadt Castrop-Rauxel | `castroprauxel.gremien.info/oparl/system` | 1.0 | more! software |

Castrop-Rauxel is deliberately kept even though it's OParl 1.0, not 1.1 —
it's a useful early edge case for the adapter (version negotiation, and a
second vendor besides SOMACOS) rather than a duplicate of the other three.

## Checked, not usable as-is

| Council | Endpoint (from `endpoints.yml`) | Result | Likely cause |
|---|---|---|---|
| Stadt Bonn | `bonn.sitzung-online.de/public/oparl/system` | Bot-check interstitial ("Zugriff prüfen") | Needs a real browser/cookie flow, not a plain GET |
| Stadt Dresden | `oparl.dresden.de/system` | HTTP 503 | Possibly transient — worth re-checking |
| Stadt Ulm | `buergerinfo.ulm.de/oparl/system` | HTTP 404 | Registry entry is stale; URL likely moved |
| Stadt Krefeld | `ris.krefeld.de/webservice/oparl/v1.1/system` | HTTP 403 | Possibly bot-blocked, not necessarily dead |
| Stadt Aachen | `ratsinfo.aachen.de/bi/oparl/1.0/system.asp` | HTTP 404 | Registry entry is stale |
| Stadt Leipzig | `ratsinformation.leipzig.de/allris_leipzig_public/oparl/system` | HTTP 500 | Server-side error, worth re-checking |
| **Landkreis Ludwigslust-Parchim** | `lwl-pch.sitzung-online.de/bi/oparl/1.0/system.asp` | DNS does not resolve | Host appears gone |

**Ludwigslust-Parchim is worth flagging outside this file too:** it's one of
the two candidate Kair trial regions named in the shared working brief
(`council-context-network.html`). Its listed OParl endpoint doesn't resolve
at all — if this region is picked as an actual pilot, its RIS/OParl access
needs re-establishing from scratch, not assumed to already work.

## Vendor landscape observed so far

Of four verified endpoints, three run **SOMACOS** (Session product) and one
runs **more! software** (more!rubin). This is too small a sample to
generalise, but it matches the expectation from earlier discussion that a
handful of RIS vendors account for most German OParl coverage — worth
tracking vendor as its own field once the survey grows, since vendor-specific
quirks (pagination, optional-field population) are a known OParl pain point.

## Next steps for this survey

- Re-check the 503/500 endpoints later — those may just be transient.
- Try the blocked ones (Bonn, Krefeld) with a real user-agent / minimal
  session handling before concluding they're unreachable, not just
  bot-gated.
- Expand beyond this hand-picked sample of ~10 candidates toward the full
  `endpoints.yml` list once the discovery crawler (not yet built — see
  main README) exists to do this at scale instead of by hand.
