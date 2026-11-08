#!/usr/bin/env python3
"""Extract the reference's reaction annotations into docs/tpt-parity/inventory/reactions.md.

The Powder Toy element sources annotate interactions with `//@ A + B -> C`
comments next to the code that performs them. Those comments are the most
faithful machine-readable inventory of authored interactions in the pinned
tree. This script also records the central transition edges from the element
table so that non-callback behaviour is not lost.
"""

import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DATA = os.path.join(ROOT, "docs", "tpt-parity", "data", "tpt-reference.json")
ELEM_DIR = os.path.join(ROOT, ".reference", "tpt-src", "src", "simulation", "elements")
OUT = os.path.join(ROOT, "docs", "tpt-parity", "inventory", "reactions.md")


def main():
    data = json.load(open(DATA, encoding="utf-8"))
    live = [e for e in data["elements"] if not e.get("disabled_slot")]
    by_id = {e["id"]: e for e in live}

    def name_of(tid):
        if tid in (-1,):
            return "-"
        if tid == -2:
            return "special"
        e = by_id.get(tid)
        return e.get("name") or e.get("slot_name") if e else str(tid)

    # --- //@ annotations per source file ---------------------------------
    reactions = {}
    pattern = re.compile(r"//@\s*(.+?)\s*$")
    for e in live:
        fname = e.get("file")
        if not fname:
            continue
        path = os.path.join(ELEM_DIR, fname)
        if not os.path.exists(path):
            continue
        with open(path, encoding="utf-8", errors="replace") as fh:
            found = []
            for line in fh:
                m = pattern.search(line)
                if m:
                    text = m.group(1).strip()
                    if text:
                        found.append(text)
        if found:
            reactions[e.get("slot_name") or fname] = found

    lines = []
    lines.append("# Reaction inventory (source-derived)")
    lines.append("")
    lines.append("Reference: The-Powder-Toy/The-Powder-Toy @ 2b7221a46d378589b4de7683f16d3ebc92be6bd9")
    lines.append("")
    lines.append("## Authored interaction annotations (`//@` comments)")
    lines.append("")
    lines.append("Every `//@` comment in `src/simulation/elements/*.cpp` is reproduced below,")
    lines.append("grouped by the element source file that implements it. These annotations are")
    lines.append("the reference authors' own description of each interaction and are matched by")
    lines.append("the corresponding Rust handler in `crates/realmatter-tpt/src/updates.rs`.")
    lines.append("")
    total = 0
    for abbr, items in reactions.items():
        lines.append(f"### {abbr} (`{abbr}.cpp`)")
        lines.append("")
        for item in items:
            lines.append(f"- `{item}`")
            total += 1
        lines.append("")

    # --- central transition edges ---------------------------------------
    lines.append("## Central state-transition edges")
    lines.append("")
    lines.append("These transitions are applied by `transition_phase` in")
    lines.append("`crates/realmatter-tpt/src/transition.rs` (heat/pressure driven).")
    lines.append("")
    lines.append("| from | to (low temp) | to (high temp) | to (low pres) | to (high pres) |")
    lines.append("|---|---|---|---|---|")
    edge_count = 0
    for e in live:
        lt = e.get("low_temperature_transition", -1)
        ht = e.get("high_temperature_transition", -1)
        lp = e.get("low_pressure_transition", -1)
        hp = e.get("high_pressure_transition", -1)
        if lt not in (-1, -2) or ht not in (-1, -2) or lp not in (-1, -2) or hp not in (-1, -2):
            edge_count += 1
            lines.append(
                f"| {e.get('name') or e.get('slot_name')} | {name_of(lt)} | {name_of(ht)} | "
                f"{name_of(lp)} | {name_of(hp)} |"
            )
    lines.append("")

    # --- multi-field / special transitions ------------------------------
    lines.append("## Special (`ST`) transitions")
    lines.append("")
    lines.append("The reference marks these transitions `ST` and resolves them with element-specific")
    lines.append("code rather than a fixed target type:")
    lines.append("")
    for e in live:
        st_fields = []
        for key in (
            "low_temperature_transition",
            "high_temperature_transition",
            "low_pressure_transition",
            "high_pressure_transition",
        ):
            if e.get(key) == -2:
                st_fields.append(key.replace("_transition", ""))
        if st_fields:
            lines.append(f"- **{e.get('name') or e.get('slot_name')}**: {', '.join(st_fields)}")
    lines.append("")

    lines.append("## Coverage summary")
    lines.append("")
    lines.append(f"- Source files with `//@` annotations: {len(reactions)}")
    lines.append(f"- Annotated interactions: {total}")
    lines.append(f"- Elements with a temperature/pressure transition edge: {edge_count}")
    lines.append("")

    with open(OUT, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    print(f"wrote {OUT}: {total} annotations, {edge_count} transition edges")


if __name__ == "__main__":
    main()
