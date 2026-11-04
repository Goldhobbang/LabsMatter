// Minimal reactive store for UI state.

const listeners = new Set();
let state = {
  tool: "brush",
  brushSize: 3,
  materialId: 2, // sand
  category: "all",
  search: "",
  playing: false,
  speed: 1,
  grid: true,
  heat: false,
  showFps: true,
  reducedMotion: false,
  highContrast: false,
  camera: { x: 0, y: 0, zoom: 1 },
  selectedMaterial: null,
  hoveredCell: null,
  status: "Ready.",
};

export function getState() { return state; }
export function setState(patch) {
  state = { ...state, ...patch };
  for (const fn of listeners) fn(state);
  return state;
}
export function subscribe(fn) { listeners.add(fn); return () => listeners.delete(fn); }
export function resetState() {
  state = { ...state, camera: { x: 0, y: 0, zoom: 1 }, playing: false };
  for (const fn of listeners) fn(state);
}
