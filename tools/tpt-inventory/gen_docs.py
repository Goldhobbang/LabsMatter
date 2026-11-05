#!/usr/bin/env python3
"""Render docs/tpt-parity/data/tpt-reference.json into markdown inventories."""

import json
import os
import sys

FLAGS = [
    ("DATA", 0x00000001),
    ("LIQ", 0x00000002),
    ("SOL", 0x00000004),
    ("GAS", 0x00000008),
    ("ENG", 0x00000010),
    ("COND", 0x00000020),
    ("PHOTP", 0x00000040),
    ("NPEN", 0x00000080),
    ("NABS", 0x00000100),
    ("NPAS", 0x00000200),
    ("DEAD", 0x00000400),
    ("HGLW", 0x00000800),
    ("LIFE", 0x00001000),
    ("RADIO", 0x00002000),
    ("LDEC", 0x00004000),
    ("LKIL", 0x00008000),
    ("LKDC", 0x00010000),
    ("SPKS", 0x00020000),
    ("NOAM", 0x00040000),
    ("NOCT", 0x00100000),
]

TRANSITION_NAMES = {
    -1: "-",
    -2: "special",
    0: "NONE",
}

SECTIONS = [
    "Walls",
    "Electronics",
    "Powered Materials",
    "Sensors",
    "Force",
    "Explosives",
    "Gases",
    "Liquids",
    "Powders",
    "Solids",
    "Radioactive",
    "Special",
    "Game Of Life",
    "Tools",
    "Favorites",
    "Decoration tools",
]


def flag_str(props):
    return "|".join(name for name, bit in FLAGS if props & bit) or "-"


def state_str(props):
    out = []
    if props & 0x1:
        out.append("powder")
    if props & 0x2:
        out.append("liquid")
    if props & 0x4:
        out.append("solid")
    if props & 0x8:
        out.append("gas")
    if props & 0x10:
        out.append("energy")
    return "/".join(out) or "-"


def fnum(v):
    if isinstance(v, float):
        if v == int(v):
            return str(int(v))
        return ("%.3f" % v).rstrip("0").rstrip(".")
    return str(v)


def main():
    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    data_path = os.path.join(root, "docs", "tpt-parity", "data", "tpt-reference.json")
    data = json.load(open(data_path, encoding="utf-8"))
    out_path = os.path.join(root, "docs", "tpt-parity", "elements.md")

    els = data["elements"]
    by_id = {e["id"]: e for e in els}
    name_by_id = {e["id"]: e.get("name") or "DISABLED" for e in els}

    def tname(v):
        if v in TRANSITION_NAMES:
            return TRANSITION_NAMES[v]
        return name_by_id.get(v, str(v))

    lines = []
    w = lines.append
    w("# LabsMatter — TPT Element Inventory")
    w("")
    w("> **Generated** by `tools/tpt-inventory/extract.py` + `gen_docs.py` from the pinned")
    w("> reference `The-Powder-Toy @ 2b7221a46d378589b4de7683f16d3ebc92be6bd9`.")
    w("> Do not hand-edit; re-run the generator instead. Machine-readable form:")
    w("> `docs/tpt-parity/data/tpt-reference.json`.")
    w("")
    live = [e for e in els if not e.get("disabled_slot")]
    enabled = [e for e in live if e.get("enabled")]
    visible = [e for e in live if e.get("menu_visible")]
    w("## Summary")
    w("")
    w("| Metric | Value |")
    w("|---|---|")
    w(f"| Element source files in `src/simulation/elements/` | {len(live)} |")
    w(f"| Element slots (IDs 0..{len(els)-1}) | {len(els)} |")
    w(f"| Disabled ID slots | {', '.join(str(e['id']) for e in els if e.get('disabled_slot')) or 'none'} |")
    w(f"| `Enabled = 1` | {len(enabled)} |")
    w(f"| `MenuVisible = 1` | {len(visible)} |")
    w(f"| Elements with a custom `Update` | {sum(1 for e in live if 'update' in e.get('functions', []))} |")
    w(f"| Elements with a custom `Graphics` | {sum(1 for e in live if 'graphics' in e.get('functions', []))} |")
    w(f"| Walls | {len(data['walls'])} |")
    w(f"| Built-in GoL rules | {len(data['gol_rules'])} |")
    w(f"| Simulation tools | {len(data['tools'])} |")
    w(f"| `pmap` type bits | {data['pmap_bits']} (`PT_NUM` = {data['pt_num']}) |")
    w("")
    w("LabsMatter tracks parity per element with the stable feature IDs `EL-<ABBR>`")
    w("(see `docs/tpt-parity/status.md`). An element only reaches `PASS` after Agent B")
    w("confirms behaviour in gameplay.")
    w("")

    # Section assignment table
    w("## Menu sections")
    w("")
    w("| # | Section | Elements (menu-visible) |")
    w("|---|---|---|")
    for sec in range(16):
        names = [e["name"] for e in live if e.get("menu_visible") and e["menu_section"] == sec]
        w(f"| {sec} | {SECTIONS[sec]} | {', '.join(names) if names else '—'} |")
    w("")
    hidden = [e["name"] for e in live if not e.get("menu_visible")]
    w(f"Elements not shown in any menu (`MenuVisible = 0`): {', '.join(hidden)}.")
    w("")

    # Full table
    w("## All elements")
    w("")
    w("Columns: `ID`, abbreviation, canonical name (TPT `Identifier`), menu section,")
    w("base colour, physical state, `Weight` (heavier sinks), `HeatConduct`,")
    w("`HeatCapacity`, `Falldown` (0 none, 1 powder, 2 liquid), flammability,")
    w("temperature/pressure transitions (element it becomes), and implemented callbacks.")
    w("")
    w("| ID | Abbr | Name | Sec | Colour | State | Weight | HC | HCp | Fall | Flam | LowT→ | HighT→ | LowP→ | HighP→ | Props | Callbacks |")
    w("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for e in live:
        sec = SECTIONS[e["menu_section"]] if e.get("menu_visible") else "hidden"
        colour = "#%06x" % e["colour"] if e.get("colour") is not None else "—"
        w(
            "| {id} | {ab} | {nm} | {sec} | {col} | {st} | {wt} | {hc} | {hcp} | {fd} | {fl} | {lt} | {ht} | {lp} | {hp} | {props} | {fns} |".format(
                id=e["id"],
                ab=e["slot_name"],
                nm=e.get("name") or "—",
                sec=sec,
                col=colour,
                st=state_str(e["properties"]),
                wt=e["weight"],
                hc=e["heat_conduct"],
                hcp=fnum(e["heat_capacity"]),
                fd=e["falldown"],
                fl=e["flammable"],
                lt=tname(e["low_temperature_transition"]),
                ht=tname(e["high_temperature_transition"]),
                lp=tname(e["low_pressure_transition"]),
                hp=tname(e["high_pressure_transition"]),
                props=flag_str(e["properties"]),
                fns=",".join(e.get("functions", [])) or "—",
            )
        )
    w("")

    # Per-element detail: transitions thresholds + description
    w("## Transition thresholds and descriptions")
    w("")
    w("| Abbr | Description | LowT ℃ | HighT ℃ | LowP | HighP |")
    w("|---|---|---|---|---|---|")
    for e in live:
        if not e.get("description"):
            continue
        w(
            "| {ab} | {d} | {lt} | {ht} | {lp} | {hp} |".format(
                ab=e["slot_name"],
                d=e["description"].replace("|", "\\|"),
                lt=fnum(e["low_temperature"]) if e["low_temperature_transition"] != -1 else "—",
                ht=fnum(e["high_temperature"]) if e["high_temperature_transition"] != -1 else "—",
                lp=fnum(e["low_pressure"]) if e["low_pressure_transition"] != -1 else "—",
                hp=fnum(e["high_pressure"]) if e["high_pressure_transition"] != -1 else "—",
            )
        )
    w("")

    with open(out_path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines))
    print("wrote", out_path, len(lines), "lines")

    # Walls + tools + GOL doc
    out2 = os.path.join(root, "docs", "tpt-parity", "world-model.md")
    m = []
    m.append("# LabsMatter — TPT Walls, Tools and GoL Rules")
    m.append("")
    m.append("> Generated from the pinned reference. Machine-readable form in")
    m.append("> `docs/tpt-parity/data/tpt-reference.json`.")
    m.append("")
    m.append("## Walls (19)")
    m.append("")
    m.append("| ID | Identifier | Name | Colour | Glow | Drawstyle | Description |")
    m.append("|---|---|---|---|---|---|---|")
    for wl in data["walls"]:
        m.append(
            "| {id} | `{ident}` | {name} | #{colour:06x} | #{eglow:06x} | {ds} | {desc} |".format(
                id=wl["id"],
                ident=wl["identifier"],
                name=wl["name"],
                ds=wl["drawstyle"],
                desc=wl["description"],
                colour=wl["colour"],
                eglow=wl["eglow"],
            )
        )
    m.append("")
    m.append("Drawstyle values are interpreted in `src/graphics/Renderer.cpp`; the")
    m.append("`drawstyle` field selects which wall texture generator runs.")
    m.append("")
    m.append("## Simulation tools (11)")
    m.append("")
    m.append("| ID | Tool | Identifier | Colour | Hooks implemented | Description |")
    m.append("|---|---|---|---|---|---|")
    for t in data["tools"]:
        m.append(
            "| {id} | {name} | `{ident}` | {col} | {hooks} | {desc} |".format(
                id=t["id"],
                name=t["name"],
                ident=t["identifier"],
                col=("#%06x" % t["colour"]) if t.get("colour") is not None else "—",
                hooks=", ".join(t["hooks"]) or "—",
                desc=t.get("description") or "—",
            )
        )
    m.append("")
    m.append("## Built-in Game of Life rules (23)")
    m.append("")
    m.append("| ID | Name | ctype | Rule bits | Stay | Begin | States | Colour | Description |")
    m.append("|---|---|---|---|---|---|---|---|---|")
    for g in data["gol_rules"]:
        m.append(
            "| {id} | {name} | `{ct}` | `0x{r:05X}` | {stay:09b} | {begin:09b} | {st} | #{c:06x} | {d} |".format(
                id=g["id"],
                name=g["name"],
                ct=g["ctype"],
                r=g["rule"],
                stay=g["stay_mask"],
                begin=g["begin_mask"],
                st=g["states"],
                c=g["colour1"],
                d=g["description"],
            )
        )
    m.append("")
    with open(out2, "w", encoding="utf-8") as fh:
        fh.write("\n".join(m))
    print("wrote", out2)


if __name__ == "__main__":
    sys.exit(main())
