//! Shared software framebuffer for LabsMatter.
//!
//! The simulation stays authoritative. This module only reads `[Cell]` and
//! writes an RGBA8 image. WebGL2, WebGPU, and Canvas2D all upload that image;
//! they do not keep a second copy of physics.
//!
//! Pixel rules follow The Powder Toy `2b7221a46d378589b4de7683f16d3ebc92be6bd9`
//! (`src/graphics/Renderer.cpp`, `src/simulation/ElementGraphics.h`):
//! flat, blend, then additive, then the persistent fire buffer. Decoration
//! runs after the base colour and before fire, when a decoration exists.
//!
//! `element_graphics` is the shared table Agent 3 overrides per element.
//! It is not a place for new element behaviour.

mod pipeline;

pub use pipeline::{mode_needs_full, paint_cell, render, view_name, RenderScratch, DISPLAY_MODES};

/// Reference `PMODE` / `FIREMODE` bits (`ElementGraphics.h`).
pub const PMODE_FLAT: u32 = 0x0000_0001;
pub const PMODE_BLOB: u32 = 0x0000_0002;
pub const PMODE_BLUR: u32 = 0x0000_0004;
pub const PMODE_GLOW: u32 = 0x0000_0008;
pub const PMODE_SPARK: u32 = 0x0000_0010;
pub const PMODE_FLARE: u32 = 0x0000_0020;
pub const PMODE_LFLARE: u32 = 0x0000_0040;
pub const PMODE_ADD: u32 = 0x0000_0080;
pub const PMODE_BLEND: u32 = 0x0000_0100;
pub const NO_DECO: u32 = 0x0000_1000;
pub const DECO_FIRE: u32 = 0x0000_2000;
pub const FIRE_ADD: u32 = 0x0001_0000;
pub const FIRE_BLEND: u32 = 0x0002_0000;
pub const FIRE_SPARK: u32 = 0x0004_0000;

/// Outputs of a TPT-style graphics callback. Inputs that this world does not
/// store are listed on [`GRAPHICS_CALLBACK_GAPS`].
#[derive(Clone, Copy, Debug)]
pub struct ElementGraphics {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    pub pixel_mode: u32,
    pub fire_a: u8,
    pub fire_r: u8,
    pub fire_g: u8,
    pub fire_b: u8,
    /// Spatial brightness jitter. Not a physical property.
    pub vary: bool,
}

/// Fields a reference graphics callback receives that this sim does not have.
/// Agent 3 cannot implement element graphics that depend on these until the
/// world grows the field. Do not invent stand-ins inside the renderer.
pub const GRAPHICS_CALLBACK_GAPS: &[&str] = &[
    "Particle.ctype / tmp / tmp2 (no per-cell auxiliary ints)",
    "Particle.life as an integer countdown separate from Cell.lifetime",
    "Particle.dcolour decoration (no decoration buffer; blend order is implemented but the source is always absent)",
    "pipeSubcallCpart / pipeSubcallTpart (no pipe payload)",
    "per-element Graphics function pointer (element_graphics is the shared table)",
    "stickman / fighter pose (no player or fighter slots)",
    "wall bmap / emap / eglow (stone is a cell, not a wall layer)",
    "air lattice pv, vx, vy, hv at CELL resolution (see AIR_FIELD_API)",
    "gravOut.forceX / forceY (see GRAVITY_FIELD_API)",
    "emp_decor",
];

/// What Agent 2 would need to expose before air display can leave the
/// particle-block approximation.
pub const AIR_FIELD_API: &str = "\
air_sample(block_x: i32, block_y: i32) -> { pv: f32, vx: f32, vy: f32, hv: f32 }
  one sample per 4x4 particle block (TPT CELL).
  pv: gauge pressure in TPT units (0 = ambient, colour clamps at ±8).
  vx, vy: air velocity in cells per frame (colour clamps at 8).
  hv: air temperature in kelvin.
  aheat_enable: bool, already approximated by the ambient flag, but without hv.";

/// Newtonian overlay and warp lensing need this. The renderer does not
/// recompute gravity.
pub const GRAVITY_FIELD_API: &str = "\
gravity_force(block_x: i32, block_y: i32) -> { fx: f32, fy: f32 }
  acceleration at the 4x4 block, same frame as the solver.
  Used by the gravity overlay and by DISPLAY_WARP lensing.
  Until it exists, the gravity view draws only the configured uniform field
  (down, off, or toward the centre) and does not bend pixels.";

/// Shared graphics description for a material id. Agent 3 replaces entries;
/// the renderer does not special-case reactions here.
pub fn element_graphics(material: u16) -> ElementGraphics {
    let (r, g, b, a) = match material {
        0 => (0, 0, 0, 0),
        1 => (128, 128, 128, 255),
        2 => (217, 194, 122, 255),
        3 => (79, 195, 247, 220),
        4 => (141, 110, 99, 230),
        5 => (176, 190, 197, 140),
        6 => (255, 111, 0, 255),
        7 => (109, 76, 65, 255),
        8 => (144, 164, 174, 255),
        9 => (255, 61, 0, 255),
        10 => (118, 255, 3, 220),
        11 => (238, 238, 238, 255),
        12 => (97, 97, 97, 120),
        13 => (179, 229, 252, 230),
        14 => (128, 222, 234, 180),
        15 => (66, 66, 66, 255),
        16 => (255, 240, 80, 255),
        17 => (230, 240, 255, 255),
        18 => (184, 115, 51, 255),
        19 => (230, 220, 80, 220),
        20 => (210, 230, 255, 90),
        21 => (180, 190, 180, 140),
        22 => (46, 140, 60, 255),
        23 => (212, 175, 55, 255),
        24 => (160, 140, 110, 255),
        25 => (90, 40, 40, 255),
        _ => (255, 0, 255, 255),
    };
    let vary = matches!(material, 2 | 3 | 4 | 10 | 11 | 15);
    let mut pixel_mode = if a == 0 {
        0
    } else if a < 250 {
        PMODE_BLEND
    } else {
        PMODE_FLAT
    };
    let mut fire_a = 0u8;
    if material == 6 {
        pixel_mode = PMODE_ADD | FIRE_ADD;
        fire_a = 255;
    } else if material == 9 {
        pixel_mode = PMODE_FLAT | FIRE_ADD;
        fire_a = 220;
    } else if material == 16 {
        pixel_mode = PMODE_ADD | PMODE_SPARK | FIRE_SPARK;
        fire_a = 255;
    }
    ElementGraphics {
        r,
        g,
        b,
        a,
        pixel_mode,
        fire_a,
        fire_r: r,
        fire_g: g,
        fire_b: b,
        vary,
    }
}
