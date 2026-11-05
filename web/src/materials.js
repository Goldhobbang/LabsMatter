// Material registry (data-driven, mirrors realmatter-core Material concept).
// Colors are sRGB hex. Categories group the palette.
import { GENERATED_MATERIALS } from "./materials.generated.js";

export const CATEGORIES = [
  { id: "powders", label: "Powders", short: "PWD" },
  { id: "liquids", label: "Liquids", short: "LIQ" },
  { id: "gases", label: "Gases", short: "GAS" },
  { id: "solids", label: "Solids", short: "SOL" },
  { id: "energy", label: "Energy", short: "NRG" },
  { id: "tools", label: "Tools", short: "TOOL" },
  { id: "all", label: "All", short: "ALL" },
];

export const MATERIALS = [
  { id: 0, abbr: "VOID", name: "Void", category: "tools", color: "#101014", density: 0, phase: "void", temp: 293, conductivity: 0, heatCap: 0, melt: null, boil: null, desc: "Erase. Right-click a material to set this as the secondary brush." },
  { id: 1, abbr: "STNE", name: "Stone", category: "solids", color: "#7a7a7a", density: 2500, phase: "solid", temp: 293, conductivity: 2.5, heatCap: 840, melt: 1473, boil: null, desc: "Inert wall. Does not melt in this build." },
  { id: 2, abbr: "SAND", name: "Sand", category: "powders", color: "#d9c27a", density: 1600, phase: "powder", temp: 293, conductivity: 0.3, heatCap: 835, melt: 1973, boil: null, desc: "Falls and piles. Dissolves in acid." },
  { id: 3, abbr: "WATR", name: "Water", category: "liquids", color: "#4fc3f7", density: 1000, phase: "liquid", temp: 293, conductivity: 0.6, heatCap: 4184, melt: 273, boil: 373, desc: "Flows. Freezes to ice and boils to steam." },
  { id: 4, abbr: "OIL", name: "Oil", category: "liquids", color: "#8d6e63", density: 900, phase: "liquid", temp: 293, conductivity: 0.15, heatCap: 2000, melt: 250, boil: 573, desc: "Floats on water and burns." },
  { id: 5, abbr: "STM", name: "Steam", category: "gases", color: "#b0bec5", density: 0.6, phase: "gas", temp: 373, conductivity: 0.025, heatCap: 2080, melt: null, boil: null, desc: "Rises. Condenses back into water." },
  { id: 6, abbr: "FIRE", name: "Fire", category: "energy", color: "#ff6f00", density: 0.3, phase: "plasma", temp: 1200, conductivity: 0.1, heatCap: 1000, melt: null, boil: null, desc: "Ignites fuel, then becomes smoke." },
  { id: 7, abbr: "WOOD", name: "Wood", category: "solids", color: "#6d4c41", density: 700, phase: "solid", temp: 293, conductivity: 0.12, heatCap: 1700, melt: null, boil: null, desc: "Burns. Acid can eat it." },
  { id: 8, abbr: "IRON", name: "Iron", category: "solids", color: "#90a4ae", density: 7874, phase: "solid", temp: 293, conductivity: 80, heatCap: 450, melt: 1811, boil: 3134, desc: "Conducts heat. Does not become a liquid here." },
  { id: 9, abbr: "LAVA", name: "Lava", category: "liquids", color: "#ff3d00", density: 3100, phase: "liquid", temp: 1500, conductivity: 1.5, heatCap: 1000, melt: 973, boil: null, desc: "Hot, dense liquid." },
  { id: 10, abbr: "ACID", name: "Acid", category: "liquids", color: "#76ff03", density: 1050, phase: "liquid", temp: 293, conductivity: 0.5, heatCap: 3500, melt: 240, boil: 360, desc: "Dissolves sand, wood, iron, salt, and gunpowder. Stone and glass resist." },
  { id: 11, abbr: "SALT", name: "Salt", category: "powders", color: "#eeeeee", density: 2160, phase: "powder", temp: 293, conductivity: 0.5, heatCap: 880, melt: 1074, boil: 1686, desc: "Heavy powder." },
  { id: 12, abbr: "SMKE", name: "Smoke", category: "gases", color: "#616161", density: 0.4, phase: "gas", temp: 400, conductivity: 0.02, heatCap: 1000, melt: null, boil: null, desc: "Rises and thins out." },
  { id: 13, abbr: "ICE", name: "Ice", category: "solids", color: "#b3e5fc", density: 917, phase: "solid", temp: 263, conductivity: 2.2, heatCap: 2100, melt: 273, boil: null, desc: "Melts into water at 273 K." },
  { id: 14, abbr: "GLAS", name: "Glass", category: "solids", color: "#80deea", density: 2500, phase: "solid", temp: 293, conductivity: 1.0, heatCap: 840, melt: 1673, boil: null, desc: "Acid-resistant solid." },
  { id: 15, abbr: "GUN", name: "Gunpowder", category: "powders", color: "#424242", density: 1000, phase: "powder", temp: 293, conductivity: 0.2, heatCap: 1000, melt: null, boil: null, desc: "Ignites and bursts into nearby powder." },
  { id: 16, abbr: "SPRK", name: "Spark", category: "energy", color: "#fff050", density: 0, phase: "energy", temp: 900, conductivity: 0, heatCap: 0, melt: null, boil: null, desc: "Travels through adjacent iron, one cell per frame, then becomes iron." },
  ...GENERATED_MATERIALS,
];

export function findMaterial(id) {
  return MATERIALS.find((m) => m.id === id) || MATERIALS[0];
}

export function searchMaterials(query, category) {
  const q = (query || "").trim().toLowerCase();
  return MATERIALS.filter((m) => {
    if (category && category !== "all" && m.category !== category) return false;
    if (!q) return true;
    return m.name.toLowerCase().includes(q) || m.abbr.toLowerCase().includes(q) || m.desc.toLowerCase().includes(q);
  });
}
