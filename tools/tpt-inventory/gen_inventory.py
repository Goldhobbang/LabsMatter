#!/usr/bin/env python3
"""Generate docs/tpt-parity/inventory/{elements,reactions,element-dependencies}.md
from docs/tpt-parity/data/tpt-reference.json."""
import json, os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DATA = os.path.join(ROOT, "docs", "tpt-parity", "data", "tpt-reference.json")
OUT = os.path.join(ROOT, "docs", "tpt-parity", "inventory")
data = json.load(open(DATA, encoding="utf-8"))
elems = data["elements"]
live = [e for e in elems if not e.get("disabled_slot")]

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

# elements.md
lines = []
lines.append("# Element inventory (generated from pinned TPT reference)")
lines.append("")
lines.append("Reference: The-Powder-Toy/The-Powder-Toy @ 2b7221a46d378589b4de7683f16d3ebc92be6bd9")
lines.append("")
lines.append(f"Total slots: {len(elems)} (live elements: {len(live)}, disabled slots: {len(elems)-len(live)})")
lines.append(f"Enabled: {sum(1 for e in live if e.get('enabled'))}, menu-visible: {sum(1 for e in live if e.get('menu_visible'))}")
lines.append("")
lines.append("| id | abbr | name | section | state flags | enabled | menu | update | graphics | create |")
lines.append("|---:|---|---|---:|---|---:|---:|---:|---:|---:|")
for e in live:
    sec = e.get("menu_section", 0)
    upd = "Y" if "update" in (e.get("functions") or []) else ""
    gfx = "Y" if "graphics" in (e.get("functions") or []) else ""
    crt = "Y" if "create" in (e.get("functions") or []) else ""
    lines.append(f"| {e['id']} | {e.get('identifier') or e.get('slot_name')} | {e.get('name') or ''} | {sec} | {flags_str(e.get('properties',0))} | {e.get('enabled',0)} | {e.get('menu_visible',0)} | {upd} | {gfx} | {crt} |")
lines.append("")
with open(os.path.join(OUT, "elements.md"), "w", encoding="utf-8") as f:
    f.write("\n".join(lines))

# element-dependencies.md
lines = []
lines.append("# Element dependency graph (generated)")
lines.append("")
lines.append("Dependencies derived from transition targets and sim_calls in the reference source.")
lines.append("")
lines.append("| id | abbr | low_temp_transition | high_temp_transition | low_pres_transition | high_pres_transition | sim_calls |")
lines.append("|---:|---|---|---|---|---|---|")
for e in live:
    abbr = e.get("identifier") or e.get("slot_name")
    lt = e.get("low_temperature_transition", -1)
    ht = e.get("high_temperature_transition", -1)
    lp = e.get("low_pressure_transition", -1)
    hp = e.get("high_pressure_transition", -1)
    calls = ",".join(e.get("sim_calls") or [])
    lines.append(f"| {e['id']} | {abbr} | {lt} | {ht} | {lp} | {hp} | {calls} |")
lines.append("")
with open(os.path.join(OUT, "element-dependencies.md"), "w", encoding="utf-8") as f:
    f.write("\n".join(lines))

# reactions.md - summary of transition-driven interactions
lines = []
lines.append("# Reaction inventory (generated summary)")
lines.append("")
lines.append("This is a structural inventory derived from the reference element table.")
lines.append("Detailed per-element interactions live in the source .cpp files; see docs/tpt-parity/elements.md.")
lines.append("")
lines.append("## State-transition edges")
lines.append("")
lines.append("| from | to (low temp) | to (high temp) | to (low pres) | to (high pres) |")
lines.append("|---|---|---|---|---|")
for e in live:
    abbr = e.get("identifier") or e.get("slot_name")
    lt = e.get("low_temperature_transition", -1)
    ht = e.get("high_temperature_transition", -1)
    lp = e.get("low_pressure_transition", -1)
    hp = e.get("high_pressure_transition", -1)
    if lt not in (-1, -2) or ht not in (-1, -2) or lp not in (-1, -2) or hp not in (-1, -2):
        def name_of(tid):
            if tid in (-1, -2):
                return "-"
            for x in live:
                if x["id"] == tid:
                    return x.get("identifier") or x.get("slot_name")
            return str(tid)
        lines.append(f"| {abbr} | {name_of(lt)} | {name_of(ht)} | {name_of(lp)} | {name_of(hp)} |")
lines.append("")
lines.append("## Elements with Update callback")
lines.append("")
upd = [e for e in live if "update" in (e.get("functions") or [])]
lines.append(f"Count: {len(upd)}")
lines.append("")
lines.append(", ".join(e.get("identifier") or e.get("slot_name") for e in upd))
lines.append("")
lines.append("## Elements with Graphics callback")
lines.append("")
gfx = [e for e in live if "graphics" in (e.get("functions") or [])]
lines.append(f"Count: {len(gfx)}")
lines.append("")
lines.append(", ".join(e.get("identifier") or e.get("slot_name") for e in gfx))
lines.append("")
with open(os.path.join(OUT, "reactions.md"), "w", encoding="utf-8") as f:
    f.write("\n".join(lines))
print("done")