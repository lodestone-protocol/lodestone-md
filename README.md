# lodestone-md (crate: mddag)

> **Implements Lodestone Protocol v2.0-draft (MD-DAG 2)**
>
> **Markdown-native DAG — heading = skeleton, links = magnetic lines,
> body = iron filings, status = list.** Zero dependencies. Deterministic.

The Rust reference implementation of the [Lodestone Protocol v2.0-draft]
(https://github.com/lodestone-protocol/lodestone-spec) — the protocol for
**conversation as experience**: AI speech is auto-DAGified into lodestones
(磁石) that absorb aligned info (iron filings) and shed noise (sand), so
context is never squeezed by redundancy.

v1.3.0 (HTML-comment JSON carrier) is **frozen** at git tag `v1.3.0`; this
branch is the v2 line, **not frozen** — it evolves until it stops changing.

## Why v2 (the magnet-ball philosophy)

> People converse like several magnet balls rolling in sand: the balls are
> main threads, the sand is redundancy and ambiguity. Balls absorb iron
> filings (aligned info), filter out sand — structure sharpens, context
> never gets crushed.

v1.3 hid metadata in HTML comments as JSON — an invisible middle layer.
v2 drops it: **markdown's own outline is the DAG**. Root `#` headings are
lodestones, `##` sub-headings are the in-ball tree, `[label](#target)`
links between lodestones are magnetic lines, and `- status:` lists carry
the state machine `draft → converged → aligned`. Humans review the raw
markdown; agents read projections on demand.

## Read protocol — choices, not free text

| Command | Level | What it gives |
|---|---|---|
| `mddag balls <file>` | L0 | one line per lodestone: title + status (+ summary if aligned) |
| `mddag ball <slug> <file>` | L1 | one lodestone: summary + sub-heading tree |
| `mddag body <slug> <file>` | L2 | body fragment (optional anchor) |
| `mddag sediment <file>` | — | sediment zone index (converged archives) |
| `mddag check <file>` | — | parse + diagnostics; exit 1 on errors |

```console
$ mddag balls session.md
# 方案选型  [converged]
# Anaphase 驾驶舱  [aligned]
  驾驶舱是 Anaphase 的界面，不是 Helix 的。
# 磁铁球哲学  [draft]

$ mddag ball anaphase-驾驶舱 session.md
# Anaphase 驾驶舱  [aligned]
  summary: 驾驶舱是 Anaphase 的界面，不是 Helix 的。
定位
命名纠错
```

## Runtime — five streaming append ops

`add-ball` / `absorb` / `advance-status` / `compress` / `append-sediment`
/ `decay` (v2.0-draft §5). Each returns the new text + an audit record line
(deterministic; the caller attaches time/source). `compress` moves an
aligned body into the `# 沉淀区` zone, leaving `- summary:` +
`[全文](#slug-full)` — the skeleton stays bounded. `decay` is the forgetting
half-ring: it removes a stale lodestone / sediment entry / line range with an
audit trail. **When** to decay is the caller's TTL policy (`DecayPolicy`:
`root_ttl`/`near_ttl`/`other_ttl`, injected, never hardcoded — 21/14/7 days
are only example values); mddag is the deterministic executor.

```rust
use mddag::{scan, ops};

let doc = scan(text);
let next = ops::advance_status(text, "方案选型").text; // draft -> converged
```

## Determinism & diagnostics

- Every public function is a pure function of input bytes; byte-identical
  input → byte-identical output.
- Diagnostics (v2.0-draft §8): `E-DUP-ID`, `E-MISSING-ID`, `E-CYCLE`,
  `E-STATUS-TRANSITION`, `E-ABSORB-ALIGNED`, `W-STATUS-MISSING`,
  `W-REF-NOT-FOUND`, `W-SELF-REF`, `W-SEDIMENT-REF`.

## Build & test

```console
cargo build --release
cargo test          # 30 tests, zero deps
```

## Repositories

- **lodestone-spec** — authoritative protocol: `spec/v2.0-draft.md`,
  `adr/ADR-0002`, `fixtures/v2/` (pure-markdown corpus).
- **lodestone-md** — this implementation (v2 line).
- v1.3 frozen assets live in the spec repo under `spec/v1.3.md`,
  `fixtures/` (25 Golden Fixtures), `registry/errors.md`.
