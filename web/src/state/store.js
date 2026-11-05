// Minimal reactive store for UI state.

const listeners = new Set();
let state = {
  tool: "brush", // brush | select
  brushRx: 4,
  brushRy: 4,
  shape: "circle",
  materialId: 2,
  secondaryId: 0,
  category: "powders",
  search: "",
  playing: false,
  speed: 1,
  grid: false,
  gridSize: 8,
  view: 0,
  showHud: true,
  showFps: true,
  camera: { x: 0, y: 0, zoom: 1 },
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
