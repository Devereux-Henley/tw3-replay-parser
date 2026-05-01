//! Debug tool: prints every BATTLE_SETUP_UNIT record's child layout
//! (index, ESF type, value) so we can identify which positional slot holds
//! a unit's veteran level. Run with:
//!
//!     cargo run --release --example dump_setup_unit -- <replay-path>
//!
//! Output format per record:
//!     === BATTLE_SETUP_UNIT [#0] ===
//!     [00] U32(123)
//!     [01] Bool(false)
//!     ...
//!
//! This file is not part of the release artefact; it's checked in only as a
//! reproducible way to re-verify field positions after a game patch.

use std::io::Cursor;

use rpfm_lib::files::{esf::ESF, Decodeable};
use serde_json::Value;

fn main() -> Result<(), String> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| "missing replay path".to_string())?;
    let bytes = std::fs::read(&path).map_err(|e| format!("read failed: {e}"))?;
    let mut cursor = Cursor::new(bytes);
    let esf = ESF::decode(&mut cursor, &None).map_err(|e| format!("decode failed: {e}"))?;
    let tree = serde_json::to_value(&esf).map_err(|e| format!("serialize failed: {e}"))?;

    let mut units = Vec::new();
    find_records(&tree, "BATTLE_SETUP_UNIT", &mut units);

    println!("found {} BATTLE_SETUP_UNIT records", units.len());
    for (i, u) in units.iter().enumerate().take(20) {
        println!("=== BATTLE_SETUP_UNIT [#{i}] ===");
        for (idx, child) in flat_children(u).iter().enumerate() {
            println!("[{idx:02}] {}", short_repr(child));
        }
    }
    Ok(())
}

fn find_records<'a>(tree: &'a Value, name: &str, out: &mut Vec<&'a Value>) {
    match tree {
        Value::Object(map) => {
            if let Some(rec) = map.get("Record") {
                if rec.get("name").and_then(Value::as_str) == Some(name) {
                    out.push(rec);
                }
            }
            for (_, v) in map {
                find_records(v, name, out);
            }
        }
        Value::Array(arr) => arr.iter().for_each(|v| find_records(v, name, out)),
        _ => {}
    }
}

fn flat_children(record: &Value) -> Vec<&Value> {
    let mut out = Vec::new();
    if let Some(groups) = record.get("children").and_then(Value::as_array) {
        for g in groups {
            if let Some(arr) = g.as_array() {
                for v in arr {
                    out.push(v);
                }
            }
        }
    }
    out
}

/// One-line summary of an ESF leaf — enough to spot integer ranks.
fn short_repr(v: &Value) -> String {
    if let Some(o) = v.as_object() {
        for (k, inner) in o {
            // Most ESF leaves are { "<TypeName>": { "value": ... } } or
            // { "<TypeName>": "<scalar>" }.
            let rendered = inner
                .get("value")
                .map(|x| x.to_string())
                .unwrap_or_else(|| inner.to_string());
            // Truncate ridiculously long arrays.
            let trimmed = if rendered.len() > 80 {
                format!("{}…", &rendered[..80])
            } else {
                rendered
            };
            return format!("{k}({trimmed})");
        }
    }
    "<non-object>".to_string()
}
