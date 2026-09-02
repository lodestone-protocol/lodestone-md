# Changelog

All notable changes to lodestone-md are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/), and this
project adheres to the Lodestone Protocol (MD-DAG) v1.3 Final.

## [1.3.0] - 2026-09-02

Initial public release — the authoritative reference implementation of the
Lodestone Protocol (MD-DAG) v1.3 Final.

### Added (P0 · parser)
- Full §8 ten-step parse pipeline: CommonMark 0.31.2 top-level node
  boundaries (comrak), document-level metadata version negotiation (§3.1),
  node metadata three-layer validation (§5: form / JSON incl. duplicate-key
  pre-scan / field degradation), E-DUP-ID without cascading (§4.2),
  derived fields `chars` / `body_start` / `body_end` (§8.1), edge
  normalization (derive transposition + folding, §7.2), deferred binding with
  `aligned`-both-ends effectiveness, and global cycle detection that
  invalidates exactly the cycle edge set (E-CYCLE, §7.3).
- CLI: `mddag <file | ->` emitting the §8 contract JSON.
- 25 Golden Fixtures covering all 14 diagnostic codes of §11 plus the §10
  worked-example figures, asserted numerically and byte-for-byte (determinism).

### Added (P1 · consumer surface)
- Three-level loading: `parse()` is the L1 skeleton; `mddag::body_text`
  provides L2 targeted body reads by node id.
- Appendix-A view projection (`projection::project`) and a human review
  surface (`projection::review`: dispute domains, dangling/cyclic edges,
  alignment debt, warning summary).
- CLI modes: `--skeleton`, `--body <id>`, `--projection`, `--review`.

### Added (P2 · ecosystem alignment)
- `tests/fixtures/MANIFEST.md`: fixture ↔ spec-section index for use as a
  cross-implementation consistency corpus (validator conformance class).
- `examples/bench.rs`: large-document health check.
- Review-decision fixtures: `slug_istanbul` (U+0130 Simple Lowercase Mapping),
  `boundary_setext` (Setext H1 is not a boundary), `meta_blank_gap`
  (metadata adopted across a blank line after the heading).
- ADR-0002 records the review memo (P2/P3/P4 false positives) and the
  deferred heuristic fast-scan decision.

### Governance
- Repository bootstrapped under the phyt-DNA methodology (`docs/`),
  ADR-0001 (Rust + comrak) and ADR-0002 (consistency corpus & performance
  strategy). Code comments in English; repository docs and communication in
  Chinese.

[1.3.0]: https://github.com/lodestone-protocol/lodestone-md/releases/tag/v1.3.0
