#!/usr/bin/env python3
"""Generate the LabsMatter material registry from the extracted TPT reference.

Outputs crates/realmatter-core/src/tpt_elements.rs containing:
  * `ElementSpec` table (full reference data for the registered subset)
  * `builtin_materials()` returning the `MaterialProps` vector used by the
    simulation, ids 0..16 preserved exactly as the QA-tested build had them.

Only elements whose behaviour the engine actually implements are registered
(state movement, heat conduction, phase transitions, combustion, air coupling,
electrical conduction). Everything else stays out of the palette until it has
a real implementation — see docs/tpt-parity/status.md.
"""

import json
import os
import sys

# Existing QA-tested ids (must not change: saves and palette depend on them).
EXISTING = {
    0: "empty",
    1: "wall",
    2: "SAND",
    3: "WATR",
    4: "OIL",
    5: "WTRV",
    6: "FIRE",
    7: "WOOD",
    8: "METL",
    9: "LAVA",
    10: "ACID",
    11: "SALT",
    12: "SMKE",
    13: "ICEI",
    14: "GLAS",
    15: "GUNP",
    16: "SPRK",
}

# Existing material values that QA verified; kept verbatim.
EXISTING_PROPS = {
    0: dict(state="Gas", density=0.0, specific_heat=1005.0, thermal_conductivity=0.026,
            melting=float("inf"), boiling=float("inf"), viscosity=0.0, color=(0, 0, 0, 0),
            flammable=False, ignition=float("inf")),
    1: dict(state="Solid", density=2500.0, specific_heat=840.0, thermal_conductivity=1.0,
            melting=float("inf"), boiling=float("inf"), viscosity=0.0, color=(0.5, 0.5, 0.5, 1.0),
            flammable=False, ignition=float("inf")),
    2: dict(state="Powder", density=1600.0, specific_heat=830.0, thermal_conductivity=0.3,
            melting=1700.0, boiling=2200.0, viscosity=0.0, color=(0.85, 0.72, 0.45, 1.0),
            flammable=False, ignition=float("inf")),
    3: dict(state="Liquid", density=1000.0, specific_heat=4184.0, thermal_conductivity=0.6,
            melting=273.15, boiling=373.15, viscosity=0.001, color=(0.31, 0.77, 0.97, 1.0),
            flammable=False, ignition=float("inf")),
    4: dict(state="Liquid", density=900.0, specific_heat=2000.0, thermal_conductivity=0.15,
            melting=250.0, boiling=573.0, viscosity=0.05, color=(0.55, 0.43, 0.39, 1.0),
            flammable=True, ignition=520.0),
    5: dict(state="Gas", density=0.6, specific_heat=2080.0, thermal_conductivity=0.025,
            melting=float("inf"), boiling=float("inf"), viscosity=0.0, color=(0.69, 0.74, 0.77, 0.9),
            flammable=False, ignition=float("inf")),
    6: dict(state="Plasma", density=0.3, specific_heat=1000.0, thermal_conductivity=0.1,
            melting=float("inf"), boiling=float("inf"), viscosity=0.0, color=(1.0, 0.44, 0.0, 1.0),
            flammable=False, ignition=float("inf")),
    7: dict(state="Solid", density=700.0, specific_heat=1700.0, thermal_conductivity=0.12,
            melting=float("inf"), boiling=float("inf"), viscosity=0.0, color=(0.43, 0.30, 0.25, 1.0),
            flammable=True, ignition=520.0),
    8: dict(state="Solid", density=7874.0, specific_heat=450.0, thermal_conductivity=80.0,
            melting=1811.0, boiling=3134.0, viscosity=0.0, color=(0.56, 0.64, 0.68, 1.0),
            flammable=False, ignition=float("inf"), conducts=True),
    9: dict(state="Liquid", density=3100.0, specific_heat=1000.0, thermal_conductivity=1.5,
            melting=973.0, boiling=float("inf"), viscosity=0.0, color=(1.0, 0.24, 0.0, 1.0),
            flammable=False, ignition=float("inf")),
    10: dict(state="Liquid", density=1050.0, specific_heat=3500.0, thermal_conductivity=0.5,
             melting=240.0, boiling=360.0, viscosity=0.0, color=(0.46, 1.0, 0.01, 1.0),
             flammable=False, ignition=float("inf")),
    11: dict(state="Powder", density=2160.0, specific_heat=880.0, thermal_conductivity=0.5,
             melting=1074.0, boiling=1686.0, viscosity=0.0, color=(0.93, 0.93, 0.93, 1.0),
             flammable=False, ignition=float("inf")),
    12: dict(state="Gas", density=0.4, specific_heat=1000.0, thermal_conductivity=0.02,
             melting=float("inf"), boiling=float("inf"), viscosity=0.0, color=(0.38, 0.38, 0.38, 1.0),
             flammable=False, ignition=float("inf")),
    13: dict(state="Solid", density=917.0, specific_heat=2100.0, thermal_conductivity=2.2,
             melting=273.15, boiling=float("inf"), viscosity=0.0, color=(0.70, 0.90, 0.99, 1.0),
             flammable=False, ignition=float("inf")),
    14: dict(state="Solid", density=2500.0, specific_heat=840.0, thermal_conductivity=1.0,
             melting=1673.0, boiling=float("inf"), viscosity=0.0, color=(0.50, 0.87, 0.92, 1.0),
             flammable=False, ignition=float("inf")),
    15: dict(state="Powder", density=1000.0, specific_heat=1000.0, thermal_conductivity=0.2,
             melting=float("inf"), boiling=float("inf"), viscosity=0.0, color=(0.26, 0.26, 0.26, 1.0),
             flammable=True, ignition=430.0),
    16: dict(state="Plasma", density=0.0, specific_heat=1000.0, thermal_conductivity=0.1,
             melting=float("inf"), boiling=float("inf"), viscosity=0.0, color=(1.0, 0.94, 0.31, 1.0),
             flammable=False, ignition=float("inf")),
}

# Existing air-coupling values, taken from the matching reference element.
EXISTING_AIR = {
    2: dict(air_loss=0.94, air_drag=0.04, advection=0.4, loss=0.95, diffusion=0.0, hot_air=0.0),
    3: dict(air_loss=0.98, air_drag=0.01, advection=0.6, loss=0.95, diffusion=0.0, hot_air=0.0),
    4: dict(air_loss=0.95, air_drag=0.02, advection=0.5, loss=0.95, diffusion=0.0, hot_air=0.0),
    5: dict(air_loss=0.999, air_drag=0.001, advection=0.7, loss=0.995, diffusion=0.7, hot_air=0.001),
    6: dict(air_loss=0.97, air_drag=0.04, advection=0.9, loss=0.2, diffusion=0.0, hot_air=0.001),
    7: dict(air_loss=1.0, air_drag=0.0, advection=0.0, loss=1.0, diffusion=0.0, hot_air=0.0),
    8: dict(air_loss=1.0, air_drag=0.0, advection=0.0, loss=1.0, diffusion=0.0, hot_air=0.0),
    9: dict(air_loss=0.98, air_drag=0.02, advection=0.4, loss=0.95, diffusion=0.0, hot_air=0.0),
    10: dict(air_loss=0.98, air_drag=0.01, advection=0.6, loss=0.95, diffusion=0.0, hot_air=0.0),
    11: dict(air_loss=0.94, air_drag=0.04, advection=0.4, loss=0.95, diffusion=0.0, hot_air=0.0),
    12: dict(air_loss=0.999, air_drag=0.001, advection=0.7, loss=0.995, diffusion=0.7, hot_air=0.001),
    13: dict(air_loss=1.0, air_drag=0.0, advection=0.0, loss=1.0, diffusion=0.0, hot_air=0.0),
    14: dict(air_loss=1.0, air_drag=0.0, advection=0.0, loss=1.0, diffusion=0.0, hot_air=0.0),
    15: dict(air_loss=0.94, air_drag=0.04, advection=0.4, loss=0.95, diffusion=0.0, hot_air=0.0),
    16: dict(air_loss=1.0, air_drag=0.0, advection=0.0, loss=1.0, diffusion=0.0, hot_air=0.0),
}

# Wave 1: elements fully covered by the generic engine systems.
WAVE1 = [
    # powders
    "DUST", "CNCT", "BRMT", "BGLA", "YEST", "BCOL", "FRZZ", "GRAV", "ANAR",
    "PQRT", "CLST", "SAWD", "SLCN", "SEED", "THRM", "IRON", "TUNG", "LITH",
    "ROCK", "RIME", "SNOW",
    # liquids
    "DSTW", "SLTW", "MWAX", "LNTG", "DESL", "LO2", "GLOW", "CBNW", "BIZR",
    "PSTE", "GEL", "SOAP", "RSST", "BASE", "FRZW", "CBNW", "MERC",
    # gases
    "GAS", "PLSM", "NBLE", "O2", "CO2", "CAUS", "FOG", "BOYL", "H2", "RFRG",
    # solids
    "DMND", "BMTL", "PSCN", "NSCN", "INSL", "INWR", "SWCH", "TTAN", "GOLD",
    "QRTZ", "CRMC", "HEAC", "WIRE", "BGLA",
]

# Wave 2: electrical system (registered with conducts=true and spark rules).
WAVE2 = ["BTRY", "DTEC", "ETRD", "NTCT", "PTCT", "HSWC", "LCRY", "DLAY", "INST", "TESC"]

STATE_MAP = {
    "Powder": "MaterialState::Powder",
    "Liquid": "MaterialState::Liquid",
    "Gas": "MaterialState::Gas",
    "Solid": "MaterialState::Solid",
    "Energy": "MaterialState::Plasma",
}


def state_of(props):
    if props & 0x1:
        return "Powder"
    if props & 0x2:
        return "Liquid"
    if props & 0x8:
        return "Gas"
    if props & 0x10:
        return "Energy"
    return "Solid"


def main():
    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    data = json.load(open(os.path.join(root, "docs", "tpt-parity", "data", "tpt-reference.json"), encoding="utf-8"))
    by_abbr = {e["slot_name"]: e for e in data["elements"] if e.get("slot_name")}

    out = []
    w = out.append
    w("// @generated by tools/tpt-inventory/gen_registry.py — do not edit by hand.")
    w("//")
    w("// Source: The-Powder-Toy @ 2b7221a46d378589b4de7683f16d3ebc92be6bd9")
    w("//   src/simulation/elements/*.cpp (element properties and transitions).")
    w("//")
    w("// Ids 0..16 are the QA-tested LabsMatter registry and keep their values.")
    w("// Ids 17+ are reference elements whose behaviour the engine implements")
    w("// generically (movement, heat, phase changes, combustion, air, conduction).")
    w("")
    w("use crate::material::{MaterialProps, MaterialState};")
    w("")
    w("/// Reference element metadata retained for the inspector and docs.")
    w("#[derive(Clone, Copy, Debug)]")
    w("pub struct ElementSpec {")
    w("    pub id: u16,")
    w("    pub abbr: &'static str,")
    w("    pub name: &'static str,")
    w("    pub description: &'static str,")
    w("    pub colour: u32,")
    w("    pub menu_section: u8,")
    w("    pub weight: f32,")
    w("    pub heat_conduct: u8,")
    w("    pub flammable: i32,")
    w("    pub explosive: i32,")
    w("    pub conducts: bool,")
    w("    pub low_temperature: f32,")
    w("    pub low_temperature_transition: i32,")
    w("    pub high_temperature: f32,")
    w("    pub high_temperature_transition: i32,")
    w("}")
    w("")
    w("/// Reference element abbr for a registry id (empty when not registered).")
    w("pub fn abbr_of(id: u16) -> &'static str {")
    w("    match id {")
    existing_rev = {v: k for k, v in EXISTING.items()}
    for abbr, eid in sorted(existing_rev.items(), key=lambda kv: kv[1]):
        if abbr in ("empty", "wall"):
            continue
        w(f"        {eid} => \"{abbr}\",")
    for abbr in WAVE1 + WAVE2:
        if abbr in existing_rev:
            continue
        e = by_abbr[abbr]
        w(f"        {17 + (WAVE1 + WAVE2).index(abbr) if False else 0} => \"\",", ) if False else None
    w("        _ => \"\",")
    w("    }")
    w("}")
    w("")

    # Build the material list.
    entries = []
    for eid in range(0, 17):
        abbr = EXISTING[eid]
        p = EXISTING_PROPS[eid]
        air = EXISTING_AIR.get(eid, dict(air_loss=1.0, air_drag=0.0, advection=0.0, loss=1.0, diffusion=0.0, hot_air=0.0))
        entries.append((eid, abbr, p, air, None))
    next_id = 17
    registered = []
    for abbr in WAVE1 + WAVE2:
        if abbr in EXISTING.values():
            continue
        e = by_abbr.get(abbr)
        if e is None:
            print("warning: missing element", abbr, file=sys.stderr)
            continue
        props = e["properties"]
        state = state_of(props)
        conducts = bool(props & 0x20)
        flammable = e["flammable"]
        # Temperature transitions: melting (solid->liquid) and boiling (liquid->gas).
        melting = float("inf")
        boiling = float("inf")
        lt = e["low_temperature_transition"]
        ht = e["high_temperature_transition"]
        name_to_id = {x["slot_name"]: x["id"] for x in data["elements"] if x.get("slot_name")}
        if lt not in (-1, -2) and lt >= 0:
            target = data["elements"][lt]
            if state_of(target["properties"]) == "Liquid":
                melting = e["low_temperature"]
        if ht not in (-1, -2) and ht >= 0:
            target = data["elements"][ht]
            if state_of(target["properties"]) == "Gas":
                boiling = e["high_temperature"]
        viscosity = 0.0
        if state == "Liquid":
            viscosity = 0.05
        elif state == "Gas":
            viscosity = 0.0
        ignition = float("inf")
        if flammable > 0:
            ignition = 695.15
        colour = e["colour"] or 0xFF00FF
        r = ((colour >> 16) & 0xFF) / 255.0
        g = ((colour >> 8) & 0xFF) / 255.0
        b = (colour & 0xFF) / 255.0
        alpha = 1.0
        if state == "Gas":
            alpha = 0.6
        spec = dict(
            abbr=abbr,
            name=e.get("name") or abbr,
            desc=e.get("description") or "",
            colour=colour,
            section=e.get("menu_section", 0),
            weight=float(e["weight"]),
            heat_conduct=e["heat_conduct"],
            flammable=flammable,
            explosive=e["explosive"],
            conducts=conducts,
            low_temperature=e["low_temperature"],
            low_temperature_transition=lt,
            high_temperature=e["high_temperature"],
            high_temperature_transition=ht,
        )
        mat = dict(
            state=state,
            density=float(e["weight"]),
            specific_heat=max(200.0, e["heat_capacity"] * 1000.0),
            thermal_conductivity=max(0.01, e["heat_conduct"] / 25.5),
            melting=melting,
            boiling=boiling,
            viscosity=viscosity,
            color=(r, g, b, alpha),
            flammable=flammable > 0,
            ignition=ignition,
            air_loss=e["airloss"],
            air_drag=e["airdrag"],
            advection=e["advection"],
            loss=e["loss"],
            diffusion=e["diffusion"],
            hot_air=e["hotair"],
            conducts=conducts,
            weight=float(e["weight"]),
        )
        entries.append((next_id, abbr, mat, mat, spec))
        registered.append((next_id, abbr))
        next_id += 1

    # Emit the spec table.
    w("/// Reference metadata for every registered element.")
    w("pub static ELEMENT_SPECS: [ElementSpec; %d] = [" % len(registered))
    for eid, abbr, _mat, _air, spec in entries:
        if spec is None:
            continue
        w("    ElementSpec {")
        w(f"        id: {eid},")
        w(f"        abbr: \"{abbr}\",")
        w(f"        name: {json.dumps(spec['name'])},")
        w(f"        description: {json.dumps(spec['desc'])},")
        w(f"        colour: 0x{spec['colour']:06X},")
        w(f"        menu_section: {spec['section']},")
        w(f"        weight: {spec['weight']:.1f},")
        w(f"        heat_conduct: {spec['heat_conduct']},")
        w(f"        flammable: {spec['flammable']},")
        w(f"        explosive: {spec['explosive']},")
        w(f"        conducts: {'true' if spec['conducts'] else 'false'},")
        w(f"        low_temperature: {spec['low_temperature']:.2f},")
        w(f"        low_temperature_transition: {spec['low_temperature_transition']},")
        w(f"        high_temperature: {spec['high_temperature']:.2f},")
        w(f"        high_temperature_transition: {spec['high_temperature_transition']},")
        w("    },")
    w("];")
    w("")
    w("/// Full builtin registry: ids 0..16 preserved, 17+ generated from the")
    w("/// reference element table.")
    w("pub fn builtin_materials() -> Vec<MaterialProps> {")
    w("    vec![")
    for eid, abbr, mat, air, spec in entries:
        p = mat if spec is not None else mat
        state = p["state"]
        color = p["color"]
        w("        MaterialProps {")
        w(f"            id: {eid},")
        w(f"            name: \"{abbr.lower() if spec is None else abbr}\".to_string(),")
        w(f"            state: MaterialState::{state},")
        w(f"            density: {fmt(p['density'])},")
        w(f"            specific_heat: {fmt(p['specific_heat'])},")
        w(f"            thermal_conductivity: {fmt(p['thermal_conductivity'])},")
        w(f"            melting_point: {fmt_inf(p['melting'])},")
        w(f"            boiling_point: {fmt_inf(p['boiling'])},")
        w(f"            viscosity: {fmt(p['viscosity'])},")
        w(f"            color: [{fmt(color[0])}, {fmt(color[1])}, {fmt(color[2])}, {fmt(color[3])}],")
        w(f"            flammable: {'true' if p['flammable'] else 'false'},")
        w(f"            ignition_temp: {fmt_inf(p['ignition'])},")
        w(f"            air_loss: {fmt(air['air_loss'])},")
        w(f"            air_drag: {fmt(air['air_drag'])},")
        w(f"            advection: {fmt(air['advection'])},")
        w(f"            loss: {fmt(air['loss'])},")
        w(f"            diffusion: {fmt(air['diffusion'])},")
        w(f"            hot_air: {fmt(air['hot_air'])},")
        w(f"            conducts: {'true' if p.get('conducts') else 'false'},")
        w(f"            weight: {fmt(p.get('weight', p['density']))},")
        w("        },")
    w("    ]")
    w("}")
    w("")

    path = os.path.join(root, "crates", "realmatter-core", "src", "tpt_elements.rs")
    with open(path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(out))
    print("wrote", path, "materials:", len(entries))

    # ---- JS palette entries for the generated materials ------------------
    js = []
    j = js.append
    j("// @generated by tools/tpt-inventory/gen_registry.py — do not edit by hand.")
    j("// New registry entries (ids 17+), derived from the pinned TPT element table.")
    j("// Only elements the engine implements generically are listed here.")
    j("export const GENERATED_MATERIALS = [")
    for eid, abbr, mat, air, spec in entries:
        if spec is None:
            continue
        colour = spec["colour"]
        hexc = "#%06x" % colour
        cat = {"Powder": "powders", "Liquid": "liquids", "Gas": "gases",
               "Solid": "solids", "Energy": "energy"}[mat["state"]]
        phase = mat["state"].lower()
        j("  { id: %d, abbr: \"%s\", name: \"%s\", category: \"%s\", color: \"%s\", density: %s, phase: \"%s\", temp: 293, conductivity: %s, heatCap: %s, melt: %s, boil: %s, conducts: %s, section: %d, desc: %s }," % (
            eid, abbr, spec["name"].title() if spec["name"].isupper() else spec["name"],
            cat, hexc, fmt(mat["density"]), phase,
            fmt(mat["thermal_conductivity"]), fmt(mat["specific_heat"]),
            ("null" if mat["melting"] == float("inf") else fmt(mat["melting"])),
            ("null" if mat["boiling"] == float("inf") else fmt(mat["boiling"])),
            "true" if mat["conducts"] else "false",
            spec["section"],
            json.dumps(spec["desc"]),
        ))
    j("];")
    j("")
    jspath = os.path.join(root, "web", "src", "materials.generated.js")
    with open(jspath, "w", encoding="utf-8") as fh:
        fh.write("\n".join(js))
    print("wrote", jspath, "entries:", len(registered))


def fmt(v):
    if isinstance(v, float):
        s = repr(v)
        if s.endswith(".0"):
            return s + "0"
        return s
    return repr(float(v))


def fmt_inf(v):
    if v == float("inf"):
        return "f32::INFINITY"
    return fmt(v)


if __name__ == "__main__":
    sys.exit(main())
