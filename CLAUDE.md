# CLAUDE.md

## What this is

A single-binary Rust CLI that reads a Total War: Warhammer 3 `.replay` file
(ESF binary format), decodes it via `rpfm_lib`, and walks the resulting JSON
tree to emit a flattened battle summary.

All logic lives in `src/main.rs`. There is no library crate, no submodules,
and currently no test suite.

## Commands

```sh
cargo build --release        # build
cargo test --release         # CI runs this; no tests exist yet
cargo run -- <replay-path>   # run against a file
```

CI (`.github/workflows/ci.yml`) runs `cargo build --release` then
`cargo test --release` on every PR and push to `main`. Releases are cut by
pushing a `v*` tag (`.github/workflows/release.yml`), which uploads the
release binary to a GitHub Release.

## Output contract

The CLI's output JSON is a public contract — it is consumed by an external
server that ingests replays. The shape is documented in `README.md`.

- The top-level document carries `schema_version` (currently `1`).
  **Bump it whenever the output shape changes in a way consumers must notice:**
  removing or renaming a field, changing a field's type, or changing the
  meaning of an existing field. Adding a new optional field is
  forward-compatible and does not require a bump.
- Errors are emitted as `{"error": "<message>"}` on stderr; success output
  goes to stdout. Keep this split — downstream tooling relies on it.
- Exit codes: `0` success, `1` runtime failure, `2` missing argument.

## How the extraction works

ESF decodes into a deeply nested `serde_json::Value` tree of `Record` nodes.
Three helpers in `main.rs` do almost all the navigation:

- `find_record(tree, name)` — first record with `name == <name>`, recursive.
- `find_records(tree, name, &mut out)` — all records with that name, recursive.
- `flat_children(record)` — flattens the record's `children` groups into a
  single ordered `Vec<&Value>`. **Order matters** — extraction reads child
  fields by positional index (e.g. `kids.get(27)` for `is_reinforcement`).

Primitive accessors (`as_u32`, `as_ascii`, `as_utf16`, `as_bool`) unwrap the
ESF type wrappers around leaf values.

### Things that are easy to break

- **Positional indices into `flat_children` are fragile.** The numbers come
  from observed ESF layouts, not a schema. If a future game patch changes
  field order in a record, the wrong fields will be read silently. Prefer
  named lookups (`find_record(record, "...")`) when adding new extractions.
- **`find_records` recurses without bounds.** Some ESF records (notably
  `BATTLE_SETUP_ARMY`) nest other records of the same kind for
  reinforcements. A naive recursive search will double-count children.
  When walking a parent record for its own descendants, write a scoped walk
  that stops descending at nested boundaries of the same record name.
- **`faction_key` heuristic** — `extract_alliance` picks the first child of
  `BATTLE_SETUP_FACTION` whose ASCII value starts with `wh`. This is a
  workaround for the faction key not being at a stable index. If you find a
  stable lookup, prefer it.

## Style

- Errors propagate as `Result<_, String>`; `format!` the underlying error
  into a short prefixed message (`"decode failed: {e}"`).
- No `unwrap` / `expect` on input-derived data. Use `.unwrap_or(...)` with a
  sensible default for missing optional fields, and `ok_or_else` to convert
  `Option` into a `Result` for fields that must exist (`BATTLE_SETUP`,
  `BATTLE_RESULTS`).
- Keep `main.rs` flat. There is no need to split into modules at this size.

## License

AGPL-3.0-or-later. Any contribution is licensed under the same terms.
