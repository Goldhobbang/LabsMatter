// Material registry (data-driven, mirrors realmatter-core Material concept).
// Colors are sRGB hex. Categories group the palette.

export const CATEGORIES = [
  { id: "all", label: "All" },
  { id: "solids", label: "Solids" },
  { id: "liquids", label: "Liquids" },
  { id: "gases", label: "Gases" },
  { id: "powders", label: "Powders" },
  { id: "special", label: "Special" },
];

export const MATERIALS = [
  { id: 0, name: "Void", category: "special", color: "#000000", density: 0, phase: "void", temp: 293, conductivity: 0, heatCap: 0, melt: null, boil: null, desc: "Empty cell." },
  { id: 1, name: "Stone", category: "solids", color: "#7a7a7a", density: 2500, phase: "solid", temp: 293, conductivity: 2.5, heatCap: 840, melt: 1473, boil: null, desc: "Inert solid rock." },
  { id: 2, name: "Sand", category: "powders", color: "#d9c27a", density: 1600, phase: "powder", temp: 293, conductivity: 0.3, heatCap: 835, melt: 1973, boil: null, desc: "Granular silica." },
  { id: 3, name: "Water", category: "liquids", color: "#4fc3f7", density: 1000, phase: "liquid", temp: 293, conductivity: 0.6, heatCap: 4184, melt: 273, boil: 373, desc: "Liquid water." },
  { id: 4, name: "Oil", category: "liquids", color: "#8d6e63", density: 900, phase: "liquid", temp: 293, conductivity: 0.15, heatCap: 2000, melt: 250, boil: 573, desc: "Flammable liquid." },
  { id: 5, name: "Steam", category: "gases", color: "#b0bec5", density: 0.6, phase: "gas", temp: 373, conductivity: 0.025, heatCap: 2080, melt: null, boil: null, desc: "Water vapor." },
  { id: 6, name: "Fire", category: "special", color: "#ff6f00", density: 0.3, phase: "plasma", temp: 1200, conductivity: 0.1, heatCap: 1000, melt: null, boil: null, desc: "Burning gas." },
  { id: 7, name: "Wood", category: "solids", color: "#6d4c41", density: 700, phase: "solid", temp: 293, conductivity: 0.12, heatCap: 1700, melt: null, boil: null, desc: "Combustible solid." },
  { id: 8, name: "Iron", category: "solids", color: "#90a4ae", density: 7874, phase: "solid", temp: 293, conductivity: 80, heatCap: 450, melt: 1811, boil: 3134, desc: "Conductive metal." },
  { id: 9, name: "Lava", category: "liquids", color: "#ff3d00", density: 3100, phase: "liquid", temp: 1500, conductivity: 1.5, heatCap: 1000, melt: 973, boil: null, desc: "Molten rock." },
  { id: 10, name: "Acid", category: "liquids", color: "#76ff03", density: 1050, phase: "liquid", temp: 293, conductivity: 0.5, heatCap: 3500, melt: 240, boil: 360, desc: "Corrosive liquid." },
  { id: 11, name: "Salt", category: "powders", color: "#eeeeee", density: 2160, phase: "powder", temp: 293, conductivity: 0.5, heatCap: 880, melt: 1074, boil: 1686, desc: "Crystalline powder." },
  { id: 12, name: "Smoke", category: "gases", color: "#616161", density: 0.4, phase: "gas", temp: 400, conductivity: 0.02, heatCap: 1000, melt: null, boil: null, desc: "Combustion byproduct." },
  { id: 13, name: "Ice", category: "solids", color: "#b3e5fc", density: 917, phase: "solid", temp: 263, conductivity: 2.2, heatCap: 2100, melt: 273, boil: null, desc: "Frozen water." },
  { id: 14, name: "Glass", category: "solids", color: "#80deea", density: 2500, phase: "solid", temp: 293, conductivity: 1.0, heatCap: 840, melt: 1673, boil: null, desc: "Transparent solid." },
  { id: 15, name: "Gunpowder", category: "powders", color: "#424242", density: 1000, phase: "powder", temp: 293, conductivity: 0.2, heatCap: 1000, melt: null, boil: null, desc: "Explosive powder." },
];

export function findMaterial(id) {
  return MATERIALS.find((m) => m.id === id) || MATERIALS[0];
}

export function searchMaterials(query, category) {
  const q = (query || "").trim().toLowerCase();
  return MATERIALS.filter((m) => {
    if (category && category !== "all" && m.category !== category) return false;
    if (!q) return true;
    return m.name.toLowerCase().includes(q) || m.desc.toLowerCase().includes(q);
  });
}
