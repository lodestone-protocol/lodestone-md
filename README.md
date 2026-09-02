# lodestone-md

> **Reference implementation of the Lodestone Protocol (MD-DAG) v1.3 Final**
> The protocol grows out of knowledge dehydration in human–AI dialogue: nodes
> act like lodestones that absorb conclusions, edges register dependencies and
> disagreements as magnetic field lines, and the state machine
> `draft → converged → aligned` marks each node's maturity.

[![CI](https://github.com/lodestone-protocol/lodestone-md/actions/workflows/ci.yml/badge.svg)](https://github.com/lodestone-protocol/lodestone-md/actions/workflows/ci.yml)

## What it is

- **The protocol** defines how to embed directed-acyclic-graph (DAG) structure
  metadata in standard Markdown via HTML comments (nodes / edges / state
  machine). Rendering is transparent, parsing is deterministic, and there are
  zero platform dependencies. Spec: Lodestone Protocol (MD-DAG) v1.3 Final.
- **This repository** is the authoritative Rust reference parser plus a Golden
  Fixture test suite. The parse output is a pure function of the document's
  current bytes (determinism first); the figures of the spec's §10 worked
  example are pinned assertion-by-assertion.

## Quick start

```bash
cargo build --release
target/release/mddag tests/fixtures/10_example.md                       # §8 contract JSON
target/release/mddag --body concl-01 tests/fixtures/10_example.md       # L2 targeted body text
target/release/mddag --projection tests/fixtures/10_example.md          # appendix-A edge labels
target/release/mddag --review tests/fixtures/10_example.md              # disputes + warning summary
```

As a library:

```rust
let result = mddag::parse(&markdown_text);
// L1 skeleton: result.nodes / result.edges / result.diagnostics / result.graph
// L2 targeted body: mddag::body_text(&markdown_text, "concl-01")
// Appendix-A projection: mddag::projection::project(&result) / mddag::projection::review(&result)
```

## Three-level loading (the protocol's native read mode)

| Level | Content | Typical use |
|---|---|---|
| L1 skeleton | node table + normalized edge set + diagnostics | rebuild global awareness, plan reads |
| L2 targeted body | read one node by `body_start` / `body_end` | answer a specific question, trace a chain |
| L3 full text | all bytes of the document | recall-complete tasks (audit / migration) |

## Tests

```bash
cargo test                        # unit + Golden Fixture + determinism assertions
UPDATE_GOLDEN=1 cargo test        # regenerate expected snapshots (review by hand)
cargo clippy --all-targets -- -D warnings   # gate: warnings denied
cargo run --release --example bench -- 5000 # coarse perf check (not a CI gate)
```

### Verification status (measured 2026-09-02)

| Gate | Measured | Verified by |
|---|---|---|
| `cargo test` | ✅ 23 passed (15 unit + 6 Golden + 2 P1 integration) | CI / local |
| `cargo clippy --all-targets -- -D warnings` | ✅ 0 warnings | CI / local |
| §10 golden baseline | ✅ chars 12/74/23/11/3, ranges 5–5/9–15/19–19/23–23/27–27, empty global graph | `tests/golden.rs::spec_10_example_values` |
| parse determinism | ✅ two parses of one input, byte-identical output | `tests/golden.rs::parse_is_deterministic` |
| diagnostic code coverage | ✅ all 14 codes of §11 (25 fixtures, one-to-one) | `tests/fixtures/MANIFEST.md` |
| L1/L2 loading semantics | ✅ `body_text()` matches derived fields char-for-char (fences, empty bodies) | `tests/p1.rs` |

## Project governance

The repository grows under the [phyt-DNA](https://github.com/Jasonmilk/phyt-DNA)
methodology; repository docs are written in Chinese and code comments in
English:

- `docs/VISION.md` — vision, atomic principles, and the seven philosophies
  (closed loop / maximal reuse / maximal decoupling / fetch on demand / load
  on demand / physical-time freshness / determinism first)
- `docs/DNA.md` — immutable principles and project-specific iron rules
- `docs/RNA.md` — three-layer loading protocol and AI collaboration rules
- `docs/PLAN.md` — growth-stage navigator (read first in a new session)
- `docs/decisions/` — ADRs (ADR-0001: Rust + comrak; ADR-0002: consistency
  corpus & performance strategy)

## License

MIT
