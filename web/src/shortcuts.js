// Configurable keyboard map. Powder preset follows The Powder Toy where a
// browser can actually honor the key. Legacy preset keeps the older LabsMatter keys.

const KEY = "labsmatter-keys-v1";

function bind(action, code, extra = {}) {
  return { action, code, ctrl: false, shift: false, alt: false, arg: null, ...extra };
}

export function powderPreset() {
  return [
    bind("pause", "Space"),
    bind("step", "KeyF"),
    bind("search", "KeyE"),
    bind("brush-shape", "Tab"),
    bind("brush-dec", "BracketLeft"),
    bind("brush-inc", "BracketRight"),
    bind("hud", "KeyH"),
    bind("grid", "KeyG"),
    bind("zoom", "KeyZ"),
    bind("view", "Digit1", { arg: 0 }),
    bind("view", "Digit2", { arg: 1 }),
    bind("view", "Digit3", { arg: 2 }),
    bind("view", "Digit4", { arg: 3 }),
    bind("view", "Digit5", { arg: 4 }),
    bind("shot", "KeyP"),
    bind("shot", "F2"),
    bind("stamp-save", "KeyS"),
    bind("stamp-load", "KeyL"),
    bind("stamp-lib", "KeyK"),
    bind("fullscreen", "F11"),
    bind("select", "KeyX"),
    bind("undo", "KeyZ", { ctrl: true }),
    bind("redo", "KeyY", { ctrl: true }),
    bind("redo", "KeyZ", { ctrl: true, shift: true }),
    bind("copy", "KeyC", { ctrl: true }),
    bind("cut", "KeyX", { ctrl: true }),
    bind("paste", "KeyV", { ctrl: true }),
    bind("help", "Slash", { shift: true }),
    bind("reset", "Backspace", { ctrl: true }),
  ];
}

export function legacyPreset() {
  return [
    bind("pause", "Space"),
    bind("step", "Period"),
    bind("tool", "KeyB", { arg: "brush" }),
    bind("tool", "KeyE", { arg: "eraser" }),
    bind("tool", "KeyL", { arg: "line" }),
    bind("tool", "KeyQ", { arg: "rect" }),
    bind("tool", "KeyF", { arg: "fill" }),
    bind("grid", "KeyG"),
    bind("heat-toggle", "KeyH"),
    bind("zoom-in", "Equal"),
    bind("zoom-out", "Minus"),
    bind("zoom-reset", "Digit0"),
    bind("reset", "KeyR"),
    bind("undo", "KeyZ", { ctrl: true }),
    bind("redo", "KeyY", { ctrl: true }),
    bind("help", "Slash", { shift: true }),
  ];
}

export const ACTION_LABELS = [
  ["pause", "Pause / resume"],
  ["step", "One frame"],
  ["search", "Focus element search"],
  ["brush-shape", "Cycle brush shape"],
  ["brush-dec", "Smaller brush"],
  ["brush-inc", "Larger brush"],
  ["hud", "Toggle HUD"],
  ["grid", "Toggle grid"],
  ["zoom", "Toggle zoom"],
  ["shot", "Save screenshot"],
  ["stamp-save", "Save stamp from selection"],
  ["stamp-load", "Place last stamp"],
  ["stamp-lib", "Stamp library"],
  ["select", "Selection tool"],
  ["undo", "Undo"],
  ["redo", "Redo"],
  ["copy", "Copy selection"],
  ["cut", "Cut selection"],
  ["paste", "Paste"],
  ["fullscreen", "Fullscreen"],
  ["reset", "Clear world (asks first)"],
  ["help", "Shortcut help"],
];

export function loadConfig() {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (parsed && Array.isArray(parsed.bindings) && (parsed.wheel === "brush" || parsed.wheel === "zoom")) {
        return parsed;
      }
    }
  } catch { /* ignore broken storage */ }
  return { preset: "powder", wheel: "brush", bindings: powderPreset() };
}

export function saveConfig(cfg) {
  localStorage.setItem(KEY, JSON.stringify(cfg));
}

export function applyPreset(name) {
  if (name === "legacy") return { preset: "legacy", wheel: "zoom", bindings: legacyPreset() };
  return { preset: "powder", wheel: "brush", bindings: powderPreset() };
}

export function matchBinding(bindings, e) {
  return bindings.find((b) =>
    b.code === e.code && !!b.ctrl === !!(e.ctrlKey || e.metaKey) && !!b.shift === !!e.shiftKey && !!b.alt === !!e.altKey
  ) || null;
}

export function formatBinding(b) {
  const parts = [];
  if (b.ctrl) parts.push("Ctrl");
  if (b.alt) parts.push("Alt");
  if (b.shift) parts.push("Shift");
  parts.push(b.code.replace(/^Key/, "").replace(/^Digit/, ""));
  return parts.join("+");
}
