// Material registry (data-driven, mirrors realmatter-core Material concept).
// Colors are sRGB hex. Categories group the palette.

export const CATEGORIES = [
  { id: "powders", label: "Powders", short: "PWD" },
  { id: "liquids", label: "Liquids", short: "LIQ" },
  { id: "gases", label: "Gases", short: "GAS" },
  { id: "solids", label: "Solids", short: "SOL" },
  { id: "energy", label: "Energy", short: "NRG" },
  { id: "elec", label: "Electronics", short: "ELC" },
  { id: "special", label: "Special", short: "SPC" },
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
  { id: 16, abbr: "SPRK", name: "Spark", category: "energy", color: "#fff050", density: 0, phase: "energy", temp: 900, conductivity: 0, heatCap: 0, melt: null, boil: null, desc: "Electricity. Sits on a conductor for several frames, then restores that conductor. Jumps one empty cell unless insulation is in the way." },
  { id: 17, abbr: "SNOW", name: "Snow", category: "powders", color: "#e6f0ff", density: 300, phase: "powder", temp: 260, conductivity: 0.15, heatCap: 2100, melt: 273, boil: null, desc: "Falls. Melts into water at 273 K." },
  { id: 18, abbr: "COPR", name: "Copper", category: "solids", color: "#b87333", density: 8960, phase: "solid", temp: 293, conductivity: 400, heatCap: 385, melt: null, boil: null, desc: "Conducts spark, same as iron." },
  { id: 19, abbr: "NITR", name: "Nitro", category: "liquids", color: "#e6dc50", density: 850, phase: "liquid", temp: 293, conductivity: 0.15, heatCap: 1700, melt: 250, boil: null, desc: "Liquid explosive. Fire sets off nearby nitro." },
  { id: 20, abbr: "HYGN", name: "Hydrogen", category: "gases", color: "#d2e6ff", density: 0.09, phase: "gas", temp: 293, conductivity: 0.18, heatCap: 14300, melt: null, boil: null, desc: "Light gas. Rises when air is on and burns." },
  { id: 21, abbr: "CO2", name: "CO2", category: "gases", color: "#b4beb4", density: 1.8, phase: "gas", temp: 293, conductivity: 0.016, heatCap: 840, melt: null, boil: null, desc: "Heavy gas. Sinks when air is on." },
  { id: 22, abbr: "PLNT", name: "Plant", category: "solids", color: "#2e8c3c", density: 400, phase: "solid", temp: 293, conductivity: 0.1, heatCap: 1700, melt: null, boil: null, desc: "Drinks adjacent water and grows. Burns." },
  { id: 23, abbr: "GOLD", name: "Gold", category: "powders", color: "#d4af37", density: 19300, phase: "powder", temp: 293, conductivity: 300, heatCap: 129, melt: 1337, boil: null, desc: "Very heavy powder." },
  { id: 24, abbr: "DUST", name: "Dust", category: "powders", color: "#a08c6e", density: 200, phase: "powder", temp: 293, conductivity: 0.05, heatCap: 1000, melt: null, boil: null, desc: "Light powder. Burns." },
  { id: 25, abbr: "BOMB", name: "Bomb", category: "solids", color: "#5a2828", density: 1500, phase: "solid", temp: 293, conductivity: 0.2, heatCap: 1000, melt: null, boil: null, desc: "At 600 K it becomes a ball of fire. Stone is left standing." },
  { id: 26, abbr: "METL", name: "Metal", category: "elec", color: "#404060", density: 7000, phase: "solid", temp: 293, conductivity: 200, heatCap: 450, melt: null, boil: null, desc: "Basic conductor. A spark holds for four frames, then the metal rests for four." },
  { id: 27, abbr: "BTRY", name: "Battery", category: "elec", color: "#858505", density: 2000, phase: "solid", temp: 293, conductivity: 200, heatCap: 450, melt: null, boil: null, desc: "Sparks nearby conductors except water, saltwater, thermistors, and insulated wire." },
  { id: 28, abbr: "PSCN", name: "P-type", category: "elec", color: "#805050", density: 2000, phase: "solid", temp: 293, conductivity: 200, heatCap: 450, melt: null, boil: null, desc: "Sends current to conductors and turns powered materials on. Does not receive from NSCN." },
  { id: 29, abbr: "NSCN", name: "N-type", category: "elec", color: "#505080", density: 2000, phase: "solid", temp: 293, conductivity: 200, heatCap: 450, melt: null, boil: null, desc: "Receives current. Does not pass it back into PSCN. Turns powered materials off." },
  { id: 30, abbr: "INST", name: "Instant", category: "elec", color: "#404039", density: 2000, phase: "solid", temp: 293, conductivity: 200, heatCap: 450, melt: null, boil: null, desc: "The whole connected wire charges in one frame. PSCN charges it, NSCN receives it. Crossed single-pixel wires do not short." },
  { id: 31, abbr: "INWR", name: "Insulated wire", category: "elec", color: "#544141", density: 2000, phase: "solid", temp: 293, conductivity: 200, heatCap: 450, melt: null, boil: null, desc: "Conducts only to PSCN, NSCN, and other insulated wire." },
  { id: 32, abbr: "WIFI", name: "Wifi", category: "elec", color: "#40a060", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "Sends a spark to wifi on the same temperature channel, one frame later." },
  { id: 33, abbr: "SWCH", name: "Switch", category: "elec", color: "#103b11", density: 2000, phase: "solid", temp: 293, conductivity: 200, heatCap: 450, melt: null, boil: null, desc: "Conducts only while on. PSCN switches it on, NSCN switches it off." },
  { id: 34, abbr: "PUMP", name: "Pump", category: "elec", color: "#0a0a3b", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "While on, shifts nearby pressure toward its temperature. PSCN on, NSCN off." },
  { id: 35, abbr: "GPMP", name: "Gravity pump", category: "elec", color: "#0a3b3b", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "While on, biases nearby mobile particles with its temperature. There is no shared gravity-field write yet." },
  { id: 36, abbr: "HSWC", name: "Heat switch", category: "elec", color: "#3b0a0a", density: 2000, phase: "solid", temp: 293, conductivity: 200, heatCap: 450, melt: null, boil: null, desc: "Equalizes heat with neighbors only while on." },
  { id: 37, abbr: "PBCN", name: "Breakable clone", category: "special", color: "#3b1d0a", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "Powered clone. PSCN enables it. Paint another material onto it to set what it copies. It disappears after repeated copies." },
  { id: 38, abbr: "LCRY", name: "Liquid crystal", category: "elec", color: "#505050", density: 2000, phase: "solid", temp: 293, conductivity: 200, heatCap: 450, melt: null, boil: null, desc: "Changes brightness when PSCN charges it and NSCN discharges it." },
  { id: 39, abbr: "PPIP", name: "Powered pipe", category: "special", color: "#444466", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "Pipe that PSCN enables and NSCN pauses. Color shows the stored material and the pipe phase." },
  { id: 40, abbr: "PIPE", name: "Pipe", category: "special", color: "#3c3c3c", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "Picks up adjacent mobile material and passes it to the next phase of pipe, then releases it." },
  { id: 41, abbr: "EMP", name: "EMP", category: "elec", color: "#66aaff", density: 2000, phase: "solid", temp: 293, conductivity: 100, heatCap: 450, melt: null, boil: null, desc: "A nearby spark starts a pulse that clears sparks and can break metal into BRMT." },
  { id: 42, abbr: "SLTW", name: "Salt water", category: "liquids", color: "#3d5cff", density: 1020, phase: "liquid", temp: 293, conductivity: 0.6, heatCap: 4000, melt: 250, boil: null, desc: "Conducts. The spark is shorter than water, then the water stays charged for 54 frames." },
  { id: 43, abbr: "RSST", name: "Resist", category: "liquids", color: "#f95b49", density: 1100, phase: "liquid", temp: 293, conductivity: 0.4, heatCap: 1500, melt: null, boil: null, desc: "Conducts once, then the resist particle is deleted at the end of the spark." },
  { id: 44, abbr: "NTCT", name: "NTC thermistor", category: "elec", color: "#505040", density: 2000, phase: "solid", temp: 293, conductivity: 200, heatCap: 450, melt: null, boil: null, desc: "Passes between PSCN and NSCN only while hotter than 100 C." },
  { id: 45, abbr: "PTCT", name: "PTC thermistor", category: "elec", color: "#405050", density: 2000, phase: "solid", temp: 293, conductivity: 200, heatCap: 450, melt: null, boil: null, desc: "Passes between PSCN and NSCN only while cooler than 100 C." },
  { id: 46, abbr: "INSL", name: "Insulation", category: "elec", color: "#6b5344", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "Blocks a spark that would otherwise jump across it. Does not conduct." },
  { id: 47, abbr: "PSTN", name: "Piston", category: "special", color: "#aaaa55", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "Draw at least two in a line. PSCN spark extends, any other spark retracts. tmp3 bits cancel a blocked move instead of shortening it." },
  { id: 48, abbr: "FRME", name: "Frame", category: "special", color: "#888888", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "A straight line of frame, perpendicular to the piston, pushes the stack in front of each cell." },
  { id: 49, abbr: "STKM", name: "Stickman", category: "special", color: "#f0d0a0", density: 500, phase: "solid", temp: 293, conductivity: 0, heatCap: 1000, melt: null, boil: null, desc: "One stickman. J left, semicolon right, I jump, O use the held element. Feet are independent of the head." },
  { id: 50, abbr: "STK2", name: "Stickman 2", category: "special", color: "#a0c0f0", density: 500, phase: "solid", temp: 293, conductivity: 0, heatCap: 1000, melt: null, boil: null, desc: "Second stickman, separate legs and element. Numpad 4/6/8/5 move, jump, and use." },
  { id: 51, abbr: "FIGH", name: "Fighter", category: "special", color: "#ffe0a0", density: 500, phase: "solid", temp: 293, conductivity: 0, heatCap: 1000, melt: null, boil: null, desc: "Chases the nearer stickman and uses its element up close. Does not attack other fighters. Paint a material onto it to set the element, or it starts with dust." },
  { id: 52, abbr: "SOAP", name: "Soap", category: "liquids", color: "#f5f5dc", density: 1000, phase: "liquid", temp: 293, conductivity: 0.5, heatCap: 4000, melt: null, boil: null, desc: "Links into a film. Deleting a middle cell stitches the neighbors. Wrap mode keeps links across the border." },
  { id: 53, abbr: "CLNE", name: "Clone", category: "special", color: "#c8e000", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "Copies nearby matter, or a material painted onto it, into an empty neighbor." },
  { id: 54, abbr: "PCLN", name: "Powered clone", category: "special", color: "#00e0c8", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "Like clone, but only while PSCN has switched it on." },
  { id: 55, abbr: "PRTI", name: "Portal in", category: "special", color: "#900090", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "Takes adjacent mobile particles into the temperature channel." },
  { id: 56, abbr: "PRTO", name: "Portal out", category: "special", color: "#700070", density: 2000, phase: "solid", temp: 293, conductivity: 0, heatCap: 450, melt: null, boil: null, desc: "Releases particles from the same temperature channel. Stickmen exit here too." },
  { id: 57, abbr: "PLUT", name: "Plutonium", category: "special", color: "#3a7a3a", density: 19000, phase: "solid", temp: 293, conductivity: 20, heatCap: 300, melt: null, boil: null, desc: "Heats when a neutron hits it and can throw another neutron aside." },
  { id: 58, abbr: "NEUT", name: "Neutron", category: "energy", color: "#80c0ff", density: 0, phase: "energy", temp: 400, conductivity: 0, heatCap: 1000, melt: null, boil: null, desc: "Moves right until it hits something. Glass and water let it through." },
  { id: 59, abbr: "PHOT", name: "Photon", category: "energy", color: "#ffff80", density: 0, phase: "energy", temp: 400, conductivity: 0, heatCap: 1000, melt: null, boil: null, desc: "Moves right. Passes glass and water. Other materials absorb it and warm up." },
  { id: 60, abbr: "ELEC", name: "Electron", category: "energy", color: "#ffff20", density: 0, phase: "energy", temp: 400, conductivity: 0, heatCap: 1000, melt: null, boil: null, desc: "Moves right and ignites the first conductor it touches." },
  { id: 61, abbr: "BRMT", name: "Broken metal", category: "elec", color: "#705040", density: 7000, phase: "solid", temp: 293, conductivity: 20, heatCap: 450, melt: null, boil: null, desc: "Metal after an EMP hit. It does not conduct." },
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
