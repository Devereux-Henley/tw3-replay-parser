# tw-replay-parser

A small Rust CLI that decodes a Total War: Warhammer 3 replay file (ESF format)
and emits a normalised JSON summary of the battle to stdout.

The output is intended for ingestion by downstream services — match listings,
army viewers, statistics — that should not have to understand the raw ESF tree.

## Build

```sh
cargo build --release
```

Requires Rust 1.94 or newer (2024 edition).

## Usage

```sh
tw-replay-parser <path-to-replay.replay>
```

On success, a JSON document is printed to stdout. On failure, a JSON object of
the form `{"error": "..."}` is printed to stderr.

### Exit codes

| Code | Meaning                              |
| ---- | ------------------------------------ |
| `0`  | Success                              |
| `1`  | Decode, extraction, or I/O failure   |
| `2`  | Missing required argument            |

## Output schema

The top-level document is versioned via `schema_version`. Consumers should
treat unknown fields as forward-compatible additions and pin to a specific
`schema_version` when shape changes matter.

```jsonc
{
  "schema_version": 1,
  "format": "<ESF signature>",
  "creation_date_unix": 0,
  "match_id": "<utf16 session id, or null>",
  "played_at": { "year": 0, "month": 0, "day": 0, "hour": 0, "minute": 0, "second": 0 },
  "victory_condition": "<ascii key, or null>",
  "uploader_local_alliance_index": 0,
  "alliances": [
    {
      "index": 0,
      "faction_key": "wh_...",
      "model_count": 0,
      "armies": [
        {
          "index": 0,
          "is_reinforcement": false,
          "commander_display": "",
          "commander_portrait": "",
          "faction_flag": "",
          "force_value": 0,
          "units": [{ "key": "wh_...", "level": 0, "cost": 0 }]
        }
      ]
    }
  ]
}
```

`uploader_local_alliance_index` is the alliance the player who saved the
replay was on. It lets a server map the uploader's identity to one specific
alliance when recording results.

Each unit's `level` is the veteran rank (0-9) the unit was bought at in
the custom-battle UI. The `unit_stats_land_experience_bonuses_tables` row
matching that level controls the cost adjustment via the engine formula
`adjusted_cost = round(base_cost * cost_multiplier) + fixed_cost`. Older
replays that predate this field still parse — consumers should treat a
missing `level` as 0.

`cost` is the engine-resolved final cost for the unit slot — base cost
adjusted for mount, mark, lore, veterancy, and any armory upgrades. This
is the value the drafting UI shows in the unit's cost pip and the number
to sum if you want a player's gold-spend total. Consumers should prefer
this over reconstructing cost from base + adder tables, since variant-key
resolution against display-name seeds is lossy for mount/mark/lore
combinations. Older parser binaries do not emit this field; consumers
should treat a missing `cost` as a signal to fall back to base-cost
lookup.

`force_value` (on the army record) is **not** the sum of unit costs —
empirically it sits well below the gold total and is likely a relative
strength or scoring index. Don't use it as a cost reference.

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE).
