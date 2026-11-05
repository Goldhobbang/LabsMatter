#!/usr/bin/env python3
"""Extract The Powder Toy element / tool / wall / GOL metadata from the pinned source tree.

Usage:
    python tools/tpt-inventory/extract.py <path-to-tpt-src> [outfile]

The reference revision is fixed by docs/tpt-parity/reference.md:
    The-Powder-Toy/The-Powder-Toy @ 2b7221a46d378589b4de7683f16d3ebc92be6bd9

The output is a JSON document that is vendored into the repository so that
LabsMatter builds do not need network access or the upstream source.
"""

import json
import os
import re
import sys

# ---------------------------------------------------------------------------
# Constant / flag tables (src/simulation/ElementDefs.h, TransitionConstants.h,
# src/SimulationConfig.h, src/simulation/MenuSection.h)
# ---------------------------------------------------------------------------

STATE_FLAGS = {
    "TYPE_PART": 0x00000001,
    "TYPE_LIQUID": 0x00000002,
    "TYPE_SOLID": 0x00000004,
    "TYPE_GAS": 0x00000008,
    "TYPE_ENERGY": 0x00000010,
    "PROP_CONDUCTS": 0x00000020,
    "PROP_PHOTPASS": 0x00000040,
    "PROP_NEUTPENETRATE": 0x00000080,
    "PROP_NEUTABSORB": 0x00000100,
    "PROP_NEUTPASS": 0x00000200,
    "PROP_DEADLY": 0x00000400,
    "PROP_HOT_GLOW": 0x00000800,
    "PROP_LIFE": 0x00001000,
    "PROP_RADIOACTIVE": 0x00002000,
    "PROP_LIFE_DEC": 0x00004000,
    "PROP_LIFE_KILL": 0x00008000,
    "PROP_LIFE_KILL_DEC": 0x00010000,
    "PROP_SPARKSETTLE": 0x00020000,
    "PROP_NOAMBHEAT": 0x00040000,
    "PROP_NOCTYPEDRAW": 0x00100000,
}

SECTIONS = {
    "SC_WALL": 0,
    "SC_ELEC": 1,
    "SC_POWERED": 2,
    "SC_SENSOR": 3,
    "SC_FORCE": 4,
    "SC_EXPLOSIVE": 5,
    "SC_GAS": 6,
    "SC_LIQUID": 7,
    "SC_POWDERS": 8,
    "SC_SOLIDS": 9,
    "SC_NUCLEAR": 10,
    "SC_SPECIAL": 11,
    "SC_LIFE": 12,
    "SC_TOOL": 13,
    "SC_FAVORITES": 14,
    "SC_DECO": 15,
}

SECTION_NAMES = [
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

CONSTANTS = {
    "CFDS": 1.0,
    "CELL": 4,
    "R_TEMP": 22.0,
    "IPL": -257.0,
    "IPH": 257.0,
    "ITL": -1.0,
    "ITH": 10000.0,
    "NT": -1,
    "MAX_TEMP": 9999.0,
    "MIN_TEMP": 0.0,
}


def parse_element_list(meson_path):
    """Return ordered list of (name, id) including disabled slots."""
    text = open(meson_path, encoding="utf-8").read()
    block = text.split("simulation_elem_names = [", 1)[1]
    block = block.split("]", 1)[0]
    tokens = re.findall(r"'([A-Za-z0-9_]+)'|(disabler\(\))", block)
    names = []
    for ident, dis in tokens:
        names.append((ident if ident else "DISABLED", dis == "disabler()"))
    out = []
    elem_id = 0
    for name, is_dis in names:
        out.append({"name": name, "id": elem_id, "disabled_slot": is_dis})
        elem_id += 1
    return out


def num(expr):
    """Evaluate a C float expression made of literals, CFDS and simple maths."""
    e = expr.strip()
    e = re.sub(r"(\d)[fF]\b", r"\1", e)
    e = e.replace("CFDS", "1.0")
    e = re.sub(r"_rgb\b", "", e)
    for k, v in CONSTANTS.items():
        e = re.sub(rf"\b{k}\b", repr(v), e)
    e = re.sub(r"\b(0[xX][0-9a-fA-F]+)", lambda m: str(int(m.group(1), 16)), e)
    # strip casts
    e = re.sub(r"\(\s*(float|int|unsigned)\s*\)", "", e)
    try:
        return eval(e, {"__builtins__": {}}, {})  # noqa: S307 - trusted local source
    except Exception:
        return None


def strip_comment(line):
    """Remove a trailing // comment that is outside of a C string literal."""
    in_str = False
    i = 0
    while i < len(line):
        ch = line[i]
        if ch == "\\" and in_str:
            i += 2
            continue
        if ch == '"':
            in_str = not in_str
        elif not in_str and ch == "/" and line[i : i + 2] == "//":
            return line[:i]
        i += 1
    return line


def parse_colour(expr):
    m = re.search(r"0[xX]([0-9a-fA-F]{6})", expr)
    if m:
        return int(m.group(1), 16)
    v = num(expr)
    if isinstance(v, (int, float)):
        return int(v) & 0xFFFFFF
    return None


def zero_defaults():
    return {
        "menu_visible": 0,
        "menu_section": 0,
        "menu_sort": 0,
        "enabled": 0,
        "advection": 0.0,
        "airdrag": 0.0,
        "airloss": 1.0,
        "loss": 1.0,
        "collision": 0.0,
        "gravity": 0.0,
        "newtonian_gravity": 1.0,
        "diffusion": 0.0,
        "hotair": 0.0,
        "falldown": 0,
        "flammable": 0,
        "explosive": 0,
        "meltable": 0,
        "hardness": 30,
        "photon_reflect": 0x3FFFFFFF,
        "weight": 50,
        "heat_conduct": 128,
        "heat_capacity": 1.0,
        "properties": STATE_FLAGS["TYPE_SOLID"],
        "carries_type_in": 0,
        "low_pressure": CONSTANTS["IPL"],
        "low_pressure_transition": -1,
        "high_pressure": CONSTANTS["IPH"],
        "high_pressure_transition": -1,
        "low_temperature": CONSTANTS["ITL"],
        "low_temperature_transition": -1,
        "high_temperature": CONSTANTS["ITH"],
        "high_temperature_transition": -1,
    }


SCALAR_KEYS = {
    "Advection": ("advection", float),
    "AirDrag": ("airdrag", float),
    "AirLoss": ("airloss", float),
    "Loss": ("loss", float),
    "Collision": ("collision", float),
    "Gravity": ("gravity", float),
    "NewtonianGravity": ("newtonian_gravity", float),
    "Diffusion": ("diffusion", float),
    "HotAir": ("hotair", float),
    "Falldown": ("falldown", int),
    "Flammable": ("flammable", int),
    "Explosive": ("explosive", int),
    "Meltable": ("meltable", int),
    "Hardness": ("hardness", int),
    "Weight": ("weight", int),
    "HeatConduct": ("heat_conduct", int),
    "HeatCapacity": ("heat_capacity", float),
    "MenuVisible": ("menu_visible", int),
    "MenuSort": ("menu_sort", int),
    "Enabled": ("enabled", int),
    "CarriesTypeIn": ("carries_type_in", int),
}

TRANSITION_KEYS = {
    "LowPressure": ("low_pressure", float),
    "HighPressure": ("high_pressure", float),
    "LowTemperature": ("low_temperature", float),
    "HighTemperature": ("high_temperature", float),
    "LowPressureTransition": ("low_pressure_transition", int),
    "HighPressureTransition": ("high_pressure_transition", int),
    "LowTemperatureTransition": ("low_temperature_transition", int),
    "HighTemperatureTransition": ("high_temperature_transition", int),
}


def parse_element_cpp(path, name_to_id):
    text = open(path, encoding="utf-8", errors="replace").read()
    # cut the constructor body: it is the first function definition
    body_start = text.find("void Element::Element_")
    if body_start < 0:
        return None
    body = text[body_start:]
    # stop at the closing brace of the definition (first line that is exactly "}")
    m = re.search(r"^\}", body, re.M)
    if m:
        body = body[: m.start()]

    el = {
        "file": os.path.basename(path),
        "identifier": None,
        "name": None,
        "colour": None,
        "description": None,
        "default_properties": {},
        "functions": [],
        "sim_calls": [],
    }
    el.update(zero_defaults())

    for raw_line in body.splitlines():
        line = strip_comment(raw_line).strip()
        m = re.match(r"Identifier\s*=\s*\"([^\"]+)\";", line)
        if m:
            el["identifier"] = m.group(1)
            continue
        m = re.match(r"Name\s*=\s*\"([^\"]+)\";", line)
        if m:
            el["name"] = m.group(1)
            continue
        m = re.match(r"Description\s*=\s*\"((?:[^\"\\]|\\.)*)\";", line)
        if m:
            el["description"] = m.group(1).replace('\\"', '"')
            continue
        m = re.match(r"Colour\s*=\s*(.+);$", line)
        if m:
            el["colour"] = parse_colour(m.group(1))
            continue
        m = re.match(r"MenuSection\s*=\s*(\w+);", line)
        if m:
            el["menu_section"] = SECTIONS.get(m.group(1), 0)
            continue
        m = re.match(r"Properties\s*=\s*(.+);$", line)
        if m:
            props = 0
            for tok in re.split(r"[|+]", m.group(1)):
                tok = tok.strip()
                if tok in STATE_FLAGS:
                    props |= STATE_FLAGS[tok]
                elif re.fullmatch(r"0[xX][0-9a-fA-F]+|\d+", tok):
                    props |= int(tok, 0)
            el["properties"] = props
            continue
        m = re.match(r"DefaultProperties\.(\w+)\s*=\s*(.+);$", line)
        if m:
            key = m.group(1)
            raw = m.group(2)
            if key == "temp":
                v = num(raw)
            else:
                v = num(raw)
            el["default_properties"][key] = v
            continue
        m = re.match(r"(\w+)\s*=\s*(.+);$", line)
        if not m:
            continue
        lhs, rhs = m.group(1), m.group(2)
        if lhs in SCALAR_KEYS:
            key, caster = SCALAR_KEYS[lhs]
            v = num(rhs)
            if v is not None:
                el[key] = caster(v)
            continue
        if lhs in TRANSITION_KEYS:
            key, caster = TRANSITION_KEYS[lhs]
            if rhs in name_to_id:
                el[key] = name_to_id[rhs]
            elif rhs == "ST":
                el[key] = -2  # special transition: element-specific code
            else:
                v = num(rhs)
                if v is not None:
                    el[key] = caster(v)
            continue
        if lhs == "PhotonReflectWavelengths":
            v = num(rhs)
            if v is not None:
                el["photon_reflect"] = int(v) & 0xFFFFFFFF
            continue

    # Which callbacks are implemented in this translation unit?
    for fn, marker in (
        ("update", "UPDATE_FUNC_ARGS"),
        ("graphics", "GRAPHICS_FUNC_ARGS"),
        ("create", "ELEMENT_CREATE_FUNC_ARGS"),
        ("create_allowed", "ELEMENT_CREATE_ALLOWED_FUNC_ARGS"),
        ("change_type", "ELEMENT_CHANGETYPE_FUNC_ARGS"),
        ("ctype_draw", "CTYPEDRAW_FUNC_ARGS"),
        ("icon", "IconGenerator ="),
    ):
        if marker in text:
            el["functions"].append(fn)
    el["functions"] = sorted(set(el["functions"]))

    # Simulation API surface used -> size of behaviour to port
    calls = sorted(
        set(
            re.findall(
                r"sim->(\w+)", text
            )
        )
    )
    el["sim_calls"] = calls
    return el


def parse_walls(simdata_path):
    text = open(simdata_path, encoding="utf-8").read()
    block = text.split("static std::vector<wall_type> LoadWalls()", 1)[1]
    block = block.split("};", 1)[0]
    walls = []
    for line in block.splitlines():
        m = re.match(
            r'\s*\{0x([0-9A-Fa-f]{6})_rgb,\s*0x([0-9A-Fa-f]{6})_rgb,\s*(\d+),\s*[\w:]+\s*,\s*String\("([^"]+)"\),\s*"([^"]+)",\s*String\("([^"]*)"\)\}',
            line,
        )
        if not m:
            continue
        walls.append(
            {
                "id": len(walls),
                "colour": int(m.group(1), 16),
                "eglow": int(m.group(2), 16),
                "drawstyle": int(m.group(3)),
                "name": m.group(4),
                "identifier": m.group(5),
                "description": m.group(6),
            }
        )
    return walls


def parse_gol(simdata_path):
    text = open(simdata_path, encoding="utf-8").read()
    block = text.split("SimulationData::builtinGol = {{", 1)[1].split("}};", 1)[0]
    rules = []
    for line in block.splitlines():
        m = re.match(
            r'\s*\{\s*"([^"]+)",\s*(GT_\w+)\s*,\s*(0x[0-9A-Fa-f]+),\s*(0x[0-9A-Fa-f]+)_rgb,\s*(0x[0-9A-Fa-f]+)_rgb,\s*NGT_\w+,\s*String\("((?:[^"\\]|\\.)*)"\)\s*\}',
            line,
        )
        if not m:
            continue
        rule = int(m.group(3), 16)
        stay = rule & 0x1FF
        begin = (rule >> 9) & 0x1FF
        states = ((rule >> 17) & 0xF) + 2
        rules.append(
            {
                "id": len(rules),
                "name": m.group(1),
                "ctype": m.group(2),
                "rule": rule,
                "stay_mask": stay,
                "begin_mask": begin,
                "states": states,
                "colour1": int(m.group(4), 16),
                "colour2": int(m.group(5), 16),
                "description": m.group(6),
            }
        )
    return rules


def parse_tools(tpt_root, name_to_id):
    meson = os.path.join(tpt_root, "src", "simulation", "simtools", "meson.build")
    text = open(meson, encoding="utf-8").read()
    block = text.split("simulation_tool_names = [", 1)[1].split("]", 1)[0]
    names = re.findall(r"'([A-Za-z0-9_]+)'", block)
    tools = []
    for idx, name in enumerate(names):
        src = os.path.join(tpt_root, "src", "simulation", "simtools", name + ".cpp")
        ident = desc = None
        colour = None
        hooks = []
        if os.path.exists(src):
            t = open(src, encoding="utf-8", errors="replace").read()
            m = re.search(r'Identifier\s*=\s*"([^"]+)"', t)
            ident = m.group(1) if m else None
            m = re.search(r'Description\s*=\s*"((?:[^"\\]|\\.)*)"', t)
            desc = m.group(1) if m else None
            m = re.search(r"Colour\s*=\s*(0x[0-9A-Fa-f]{6})_rgb", t)
            colour = int(m.group(1), 16) if m else None
            for hook in (
                "Perform",
                "PerformClick",
                "PerformDrag",
                "PerformDraw",
                "PerformDrawLine",
                "PerformDrawRect",
                "PerformDrawFill",
                "PerformSelect",
            ):
                if re.search(rf"{hook}\s*=", t):
                    hooks.append(hook)
        tools.append(
            {
                "id": idx,
                "name": name,
                "identifier": ident,
                "colour": colour,
                "description": desc,
                "hooks": hooks,
            }
        )
    return tools


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    tpt_root = sys.argv[1]
    outfile = sys.argv[2] if len(sys.argv) > 2 else "docs/tpt-parity/data/tpt-reference.json"

    elem_dir = os.path.join(tpt_root, "src", "simulation", "elements")
    listing = parse_element_list(os.path.join(elem_dir, "meson.build"))
    name_to_id = {e["name"]: e["id"] for e in listing}
    name_to_id.update({"PT_" + k: v for k, v in name_to_id.items()})

    elements = []
    for entry in listing:
        if entry["disabled_slot"]:
            elements.append(
                {
                    "id": entry["id"],
                    "name": "DISABLED",
                    "disabled_slot": True,
                    "identifier": None,
                }
            )
            continue
        path = os.path.join(elem_dir, entry["name"] + ".cpp")
        parsed = parse_element_cpp(path, name_to_id)
        if parsed is None:
            raise SystemExit("failed to parse " + path)
        parsed["id"] = entry["id"]
        parsed["slot_name"] = entry["name"]
        parsed["disabled_slot"] = False
        elements.append(parsed)

    simdata = os.path.join(tpt_root, "src", "simulation", "SimulationData.cpp")
    data = {
        "reference": {
            "repository": "The-Powder-Toy/The-Powder-Toy",
            "commit": "2b7221a46d378589b4de7683f16d3ebc92be6bd9",
            "element_sources": sum(1 for e in elements if not e["disabled_slot"]),
        },
        "sections": [
            {"id": i, "name": n, "icon": icon} for i, (n, icon) in enumerate(
                zip(SECTION_NAMES, ["E041", "E042", "E056", "E019", "E062", "E043", "E045", "E044",
                                    "E050", "E051", "E046", "E04C", "E052", "E057", "E067", "E064"])
            )
        ],
        "elements": elements,
        "walls": parse_walls(simdata),
        "gol_rules": parse_gol(simdata),
        "tools": parse_tools(tpt_root, name_to_id),
        "pmap_bits": 9,
        "pt_num": 1 << 9,
    }

    os.makedirs(os.path.dirname(outfile) or ".", exist_ok=True)
    with open(outfile, "w", encoding="utf-8") as fh:
        json.dump(data, fh, indent=1, sort_keys=False)
    live = [e for e in elements if not e["disabled_slot"]]
    enabled = [e for e in live if e.get("enabled")]
    visible = [e for e in live if e.get("menu_visible")]
    print(
        f"elements={len(elements)} live={len(live)} enabled={len(enabled)} "
        f"menu_visible={len(visible)} walls={len(data['walls'])} "
        f"gol={len(data['gol_rules'])} tools={len(data['tools'])}"
    )
    print("wrote", outfile)
    return 0


if __name__ == "__main__":
    sys.exit(main())
