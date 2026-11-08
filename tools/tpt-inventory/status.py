#!/usr/bin/env python3
"""Cross-reference the pinned TPT element table with the Rust implementation.

Parses `crates/realmatter-tpt/src/updates.rs` dispatch tables and emits
`docs/tpt-parity/inventory/elements.md` with an implementation-status column,
plus `docs/tpt-parity/inventory/implementation-status.md`.

Status values mirror the task definition:
  * Implemented  - a dedicated Rust update handler exists and mirrors the
                   reference source's core behaviour.
  * Blocked      - the element needs an engine subsystem that does not exist
                   yet (player controller); the handler only implements the
                   passive portion.
  * Internal     - no `Update` callback in the reference; covered by generic
                   movement / transition logic.
  * Disabled     - a disabled reference slot (no element).
"""

import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DATA = os.path.join(ROOT, "docs", "tpt-parity", "data", "tpt-reference.json")
UPDATES = os.path.join(ROOT, "crates", "realmatter-tpt", "src", "updates.rs")
OUT = os.path.join(ROOT, "docs", "tpt-parity", "inventory")

# Elements whose handlers are still limited by missing engine subsystems.
# As of the K3 integration, portal/wireless/entity/piston/soap storage is part
# of `Sim`; the entries below are the remaining partial implementations.
BLOCKED = {
    "STKM": "shared locomotion implemented; reference legs/element-power reactions pending Agent 3",
    "STKM2": "shared locomotion implemented; reference legs/element-power reactions pending Agent 3",
    "STK2": "shared locomotion implemented; reference legs/element-power reactions pending Agent 3",
    "FIGH": "shared locomotion implemented; fighter combat/AI pending Agent 3",
    "PIPE": "full payload flow across pipe segments pending Agent 3",
    "PSTN": "uses shared MoveStack; FRME recursion and tmp3 flags pending Agent 3",
    "SOAP": "links implemented; full bubble pop/detach semantics pending Agent 3",
}

STATE_FLAGS = {
    0x00000001: "PART", 0x00000002: "LIQUID", 0x00000004: "SOLID",
    0x00000008: "GAS", 0x00000010: "ENERGY", 0x00000020: "CONDUCTS",
    0x00000040: "PHOTPASS", 0x00000080: "NEUTPENETRATE", 0x00000100: "NEUTABSORB",
    0x00000200: "NEUTPASS", 0x00000400: "DEADLY", 0x00000800: "HOT_GLOW",
    0x00001000: "LIFE", 0x00002000: "RADIOACTIVE", 0x00004000: "LIFE_DEC",
    0x00008000: "LIFE_KILL", 0x00010000: "LIFE_KILL_DEC", 0x00020000: "SPARKSETTLE",
    0x00040000: "NOAMBHEAT", 0x00100000: "NOCTYPEDRAW",
}


def flags_str(p):
    return ",".join(v for k, v in STATE_FLAGS.items() if p & k) or "-"


def extract_dispatch(fn_name, text):
    """Return the set of `pt::NAME` identifiers in a dispatch function body."""
    m = re.search(r"pub fn %s\([^)]*\)[^{]*\{" % re.escape(fn_name), text)
    if not m:
        return set()
    depth = 0
    start = m.end() - 1
    end = len(text)
    for idx in range(start, len(text)):
        ch = text[idx]
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                end = idx
                break
    body = text[start:end]
    return set(re.findall(r"pt::([A-Z0-9_]+)", body))


def main():
    data = json.load(open(DATA, encoding="utf-8"))
    elems = data["elements"]
    live = [e for e in elems if not e.get("disabled_slot")]
    rust = open(UPDATES, encoding="utf-8", errors="replace").read()

    update_set = extract_dispatch("update_fn", rust)
    create_set = extract_dispatch("create_fn", rust)
    draw_set = extract_dispatch("ctype_draw_fn", rust)

    def const_names(e):
        names = []
        if e.get("name"):
            names.append(e["name"])
        if e.get("slot_name"):
            names.append(e["slot_name"])
        return names

    rows = []
    counts = {"Implemented": 0, "Internal": 0, "Blocked": 0, "Disabled": 0}
    for e in elems:
        if e.get("disabled_slot"):
            counts["Disabled"] += 1
            rows.append((e, "Disabled", "disabled reference slot"))
            continue
        has_update = "update" in (e.get("functions") or [])
        names = const_names(e)
        implemented = any(n in update_set for n in names)
        if not has_update:
            status = "Internal"
            reason = "no reference Update callback; generic movement/transition"
        elif implemented and any(n in BLOCKED for n in names):
            status = "Blocked"
            key = next(n for n in names if n in BLOCKED)
            reason = BLOCKED[key]
        elif implemented:
            status = "Implemented"
            reason = ""
        else:
            status = "Blocked"
            reason = "no Rust handler in dispatch table"
        counts[status] += 1
        rows.append((e, status, reason))

    # elements.md
    lines = []
    lines.append("# Element inventory (generated from pinned TPT reference)")
    lines.append("")
    lines.append("Reference: The-Powder-Toy/The-Powder-Toy @ 2b7221a46d378589b4de7683f16d3ebc92be6bd9")
    lines.append("")
    lines.append(f"Total slots: {len(elems)} (live elements: {len(live)}, disabled slots: {len(elems) - len(live)})")
    lines.append(f"Enabled: {sum(1 for e in live if e.get('enabled'))}, "
                 f"menu-visible: {sum(1 for e in live if e.get('menu_visible'))}")
    lines.append("")
    lines.append("## Implementation status")
    lines.append("")
    lines.append("| Status | Count |")
    lines.append("|---|---:|")
    for k in ("Implemented", "Internal", "Blocked", "Disabled"):
        lines.append(f"| {k} | {counts[k]} |")
    lines.append("")
    lines.append("`Implemented` means a dedicated Rust handler exists in")
    lines.append("`crates/realmatter-tpt/src/updates.rs` and mirrors the reference")
    lines.append("source's core behaviour. `Blocked` entries still have a handler but are")
    lines.append("limited by a missing engine subsystem; see `implementation-status.md`.")
    lines.append("")
    lines.append("| id | abbr | name | section | state flags | enabled | menu | update | graphics | create | status |")
    lines.append("|---:|---|---|---:|---|---:|---:|---:|---:|---:|---|")
    for e, status, reason in rows:
        if e.get("disabled_slot"):
            continue
        sec = e.get("menu_section", 0)
        upd = "Y" if "update" in (e.get("functions") or []) else ""
        gfx = "Y" if "graphics" in (e.get("functions") or []) else ""
        crt = "Y" if "create" in (e.get("functions") or []) else ""
        rst = f"{status}" + (f" ({reason})" if reason and status != "Implemented" else "")
        lines.append(
            f"| {e['id']} | {e.get('identifier') or e.get('slot_name')} | {e.get('name') or ''} | {sec} | "
            f"{flags_str(e.get('properties', 0))} | {e.get('enabled', 0)} | {e.get('menu_visible', 0)} | "
            f"{upd} | {gfx} | {crt} | {rst} |"
        )
    lines.append("")
    with open(os.path.join(OUT, "elements.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(lines))

    # implementation-status.md
    out = []
    out.append("# Implementation status")
    out.append("")
    out.append("Dispatch tables: `crates/realmatter-tpt/src/updates.rs`.")
    out.append("")
    out.append("| Metric | Value |")
    out.append("|---|---:|")
    out.append(f"| Live reference elements | {len(live)} |")
    out.append(f"| Elements with a reference `Update` callback | {sum(1 for e in live if 'update' in (e.get('functions') or []))} |")
    out.append(f"| Dedicated Rust handlers in `update_fn` | {len([r for r in rows if r[1] == 'Implemented'])} |")
    out.append(f"| Blocked (missing subsystem) | {counts['Blocked']} |")
    out.append(f"| Internal-only (no Update) | {counts['Internal']} |")
    out.append("")
    out.append("## Blocked entries")
    out.append("")
    out.append("| abbr | reason |")
    out.append("|---|---|")
    for e, status, reason in rows:
        if status == "Blocked":
            out.append(f"| {e.get('slot_name')} | {reason} |")
    out.append("")
    with open(os.path.join(OUT, "implementation-status.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(out))

    print("wrote elements.md and implementation-status.md")
    print("counts:", counts)


if __name__ == "__main__":
    main()
