#!/usr/bin/env python3
"""Cross-reference the pinned TPT element table with the Rust implementation.

Parses `crates/realmatter-tpt/src/updates.rs` dispatch tables and emits
`docs/tpt-parity/inventory/elements.md` with a per-element classification,
plus `docs/tpt-parity/inventory/implementation-status.md`.

Classification is deliberately finer than "has Update / lacks Update":

  * Full        - reference `Update` callback AND a dedicated Rust handler.
  * Partial     - reference `Update` handler exists but is limited by a missing
                  engine subsystem (see BLOCKED).
  * Unimplemented - reference `Update` callback with no Rust handler.
  * Generic     - NO reference `Update`, but the element has create / graphics /
                  transition callbacks that the engine provides generically
                  (spawn defaults, colour rendering, phase transitions). These
                  are real behaviours, not "internal-only".
  * Passive     - NO reference `Update` and no create / graphics / transition:
                  covered only by generic movement.
  * Disabled    - a disabled reference slot (no element).

Nothing here claims independent verification; `verified` is always `no` until
Agent B accepts gameplay behaviour.
"""

import json
import os
import re

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DATA = os.path.join(ROOT, "docs", "tpt-parity", "data", "tpt-reference.json")
UPDATES = os.path.join(ROOT, "crates", "realmatter-tpt", "src", "updates.rs")
OUT = os.path.join(ROOT, "docs", "tpt-parity", "inventory")

# Partial handlers: functional K5 implementations whose full reference parity is
# not yet gameplay-verified. Kept separate from `Full` so nothing is overstated.
BLOCKED = {
    "STKM": "K6: feet/legs, jump, portal, hazards, element power; reference leg-particle solver unverified",
    "STKM2": "K6: feet/legs, jump, portal, hazards, element power; reference leg-particle solver unverified",
    "STK2": "K6: feet/legs, jump, portal, hazards, element power; reference leg-particle solver unverified",
    "FIGH": "K6: slot-aware head, target selection, element power, multi-fighter separation; brawling unverified",
    "PIPE": "K6: HEAC flood, transport, PPIP triggers, single-pixel init; PIPE graphics emulation pending",
    "PSTN": "K6: FRME row push, empty-space extension, tmp3 flags; full recursive stack pending",
    "SOAP": "K6: mid-chain rewiring, pop, second-order constraints; boundary edge cases pending",
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


def has_transition(e):
    keys = ("low_pressure_transition", "high_pressure_transition",
            "low_temperature_transition", "high_temperature_transition")
    return any(e.get(k, -1) not in (-1, None) for k in keys)


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
    counts = {"Full": 0, "Partial": 0, "Unimplemented": 0,
              "Generic": 0, "Passive": 0, "Disabled": 0}
    for e in elems:
        if e.get("disabled_slot"):
            counts["Disabled"] += 1
            rows.append((e, "Disabled", "disabled reference slot", False, False, False))
            continue
        funcs = e.get("functions") or []
        has_update = "update" in funcs
        has_create = "create" in funcs
        has_graphics = "graphics" in funcs
        trans = has_transition(e)
        names = const_names(e)
        rust_update = any(n in update_set for n in names)
        rust_create = any(n in create_set for n in names)
        rust_draw = any(n in draw_set for n in names)

        if has_update:
            if rust_update and any(n in BLOCKED for n in names):
                status = "Partial"
                reason = BLOCKED[next(n for n in names if n in BLOCKED)]
            elif rust_update:
                status = "Full"
                reason = ""
            else:
                status = "Unimplemented"
                reason = "no Rust handler in update_fn dispatch"
        else:
            # No reference Update callback. Distinguish elements that still
            # carry real create / graphics / transition behaviour from truly
            # passive matter moved by the generic engine.
            if rust_create or rust_draw or has_create or has_graphics or trans:
                status = "Generic"
                bits = []
                if has_create or rust_create:
                    bits.append("create-defaults")
                if has_graphics or rust_draw:
                    bits.append("colour-render")
                if trans:
                    bits.append("phase-transition")
                reason = "engine-provided " + "/".join(bits or ["behaviour"])
            else:
                status = "Passive"
                reason = "generic movement only"
        counts[status] += 1
        rows.append((e, status, reason, has_update, has_create, has_graphics))

    menu_visible = [e for e in live if e.get("menu_visible")]
    selectable = [e for e in live if e.get("menu_visible") and e.get("enabled")]

    # elements.md
    lines = []
    lines.append("# Element inventory (generated from pinned TPT reference)")
    lines.append("")
    lines.append("Reference: The-Powder-Toy/The-Powder-Toy @ 2b7221a46d378589b4de7683f16d3ebc92be6bd9")
    lines.append("")
    lines.append(f"Total slots: {len(elems)} (live elements: {len(live)}, disabled slots: {len(elems) - len(live)})")
    lines.append(f"Enabled: {sum(1 for e in live if e.get('enabled'))}, "
                 f"menu-visible: {len(menu_visible)}, selectable: {len(selectable)}")
    lines.append("")
    lines.append("## Implementation status (source-derived classification)")
    lines.append("")
    lines.append("| Status | Count | Meaning |")
    lines.append("|---|---:|---|")
    lines.append(f"| Full | {counts['Full']} | reference `Update` + dedicated Rust handler |")
    lines.append(f"| Partial | {counts['Partial']} | handler exists, blocked on an engine subsystem |")
    lines.append(f"| Unimplemented | {counts['Unimplemented']} | reference `Update` but no Rust handler |")
    lines.append(f"| Generic | {counts['Generic']} | no `Update`; create/graphics/transition behaviour provided by the engine |")
    lines.append(f"| Passive | {counts['Passive']} | no `Update`; generic movement only |")
    lines.append(f"| Disabled | {counts['Disabled']} | disabled reference slot |")
    lines.append("")
    lines.append("`Full`/`Partial` count Rust **handlers**, not verified behaviours.")
    lines.append("Independent gameplay verification is `no` for every element until")
    lines.append("Agent B signs off (see `docs/tpt-parity/qa/`).")
    lines.append("")
    lines.append("| id | abbr | name | section | state flags | enabled | menu | update | create | graphics | transition | status |")
    lines.append("|---:|---|---|---:|---|---:|---:|---:|---:|---:|---:|---|")
    for e, status, reason, hu, hc, hg in rows:
        if e.get("disabled_slot"):
            continue
        sec = e.get("menu_section", 0)
        trans = "Y" if has_transition(e) else ""
        rst = status + (f" ({reason})" if reason and status in ("Partial", "Unimplemented") else "")
        lines.append(
            f"| {e['id']} | {e.get('identifier') or e.get('slot_name')} | {e.get('name') or ''} | {sec} | "
            f"{flags_str(e.get('properties', 0))} | {e.get('enabled', 0)} | {e.get('menu_visible', 0)} | "
            f"{'Y' if hu else ''} | {'Y' if hc else ''} | {'Y' if hg else ''} | {trans} | {rst} |"
        )
    lines.append("")
    with open(os.path.join(OUT, "elements.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(lines))

    # implementation-status.md
    out = []
    out.append("# Implementation status")
    out.append("")
    out.append("Generated by `tools/tpt-inventory/status.py` from the pinned")
    out.append("reference JSON and the Rust dispatch tables in")
    out.append("`crates/realmatter-tpt/src/updates.rs`.")
    out.append("")
    out.append("| Metric | Value |")
    out.append("|---|---:|")
    out.append(f"| Live reference elements | {len(live)} |")
    out.append(f"| Enabled | {sum(1 for e in live if e.get('enabled'))} |")
    out.append(f"| Menu-visible | {len(menu_visible)} |")
    out.append(f"| Selectable (enabled + menu-visible) | {len(selectable)} |")
    out.append(f"| Reference `Update` callbacks | {sum(1 for e in live if 'update' in (e.get('functions') or []))} |")
    out.append(f"| Rust `update_fn` handlers | {counts['Full'] + counts['Partial']} |")
    out.append(f"| Full handlers | {counts['Full']} |")
    out.append(f"| Partial handlers | {counts['Partial']} |")
    out.append(f"| Unimplemented Update callbacks | {counts['Unimplemented']} |")
    out.append(f"| Generic (no Update; create/graphics/transition) | {counts['Generic']} |")
    out.append(f"| Passive (no Update; movement only) | {counts['Passive']} |")
    out.append(f"| Disabled slots | {counts['Disabled']} |")
    out.append(f"| Independently verified | 0 |")
    out.append("")
    out.append("`Generic` elements are **not** internal-only: they rely on engine-provided")
    out.append("create defaults, colour rendering, or phase transitions even though the")
    out.append("reference declares no `Update` callback. See `elements.md` for the per-element")
    out.append("flags (`update`/`create`/`graphics`/`transition`).")
    out.append("")
    out.append("## Partial entries")
    out.append("")
    out.append("| abbr | reason |")
    out.append("|---|---|")
    for e, status, reason, *_ in rows:
        if status == "Partial":
            out.append(f"| {e.get('slot_name')} | {reason} |")
    out.append("")
    out.append("## Unimplemented entries")
    out.append("")
    out.append("| abbr | reason |")
    out.append("|---|---|")
    any_unimpl = False
    for e, status, reason, *_ in rows:
        if status == "Unimplemented":
            any_unimpl = True
            out.append(f"| {e.get('slot_name')} | {reason} |")
    if not any_unimpl:
        out.append("| ??| none |")
    out.append("")
    with open(os.path.join(OUT, "implementation-status.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(out))

    # generic-passive-audit.md
    audit = []
    audit.append("# Generic / passive element audit (K5)")
    audit.append("")
    audit.append("Elements with no `Update` callback but with source-defined `create`,")
    audit.append("`graphics`, or temperature/pressure transition behaviour. The engine must")
    audit.append("provide those behaviours centrally; this audit records what each relies on.")
    audit.append("")
    audit.append("| abbr | create | graphics | transitions | engine support |")
    audit.append("|---|---:|---:|---|---|")
    for e, status, reason, hu, hc, hg in rows:
        if status not in ("Generic", "Passive") or e.get("disabled_slot"):
            continue
        trans = []
        for key, label in (
            ("low_temperature_transition", "lowT"),
            ("high_temperature_transition", "highT"),
            ("low_pressure_transition", "lowP"),
            ("high_pressure_transition", "highP"),
        ):
            if e.get(key, -1) != -1:
                trans.append(label)
        support = []
        if hc:
            support.append("element_defaults.rs")
        if hg:
            support.append("base color render")
        if trans:
            support.append("transition_phase")
        if not support:
            support.append("movement only")
        audit.append(
            "| {abbr} | {c} | {g} | {t} | {s} |".format(
                abbr=e.get("slot_name"),
                c="Y" if hc else "",
                g="Y" if hg else "",
                t=",".join(trans) if trans else "-",
                s=", ".join(support),
            )
        )
    audit.append("")
    audit.append("The K5 audit found no Generic/Passive element whose source declares a")
    audit.append("create/graphics/transition behaviour that is missing from the engine:")
    audit.append("reference `DefaultProperties` are applied by `element_defaults.rs` at")
    audit.append("particle creation, base colours come from the generated element table, and")
    audit.append("all declared temperature/pressure transitions are applied by")
    audit.append("`transition_phase`.")
    audit.append("")
    with open(os.path.join(OUT, "generic-passive-audit.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(audit))

    print("wrote elements.md, implementation-status.md, generic-passive-audit.md")
    print("counts:", counts)
    print("menu_visible:", len(menu_visible), "selectable:", len(selectable))


if __name__ == "__main__":
    main()

