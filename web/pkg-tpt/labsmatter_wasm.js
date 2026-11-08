/**
 * Simulation tools, mirroring `realmatter-wasm`:
 * 0 heat, 1 cool, 2 air (pressure+), 3 vacuum, 4 PGRV, 5 NGRV, 6 mix,
 * 7 cyclone, 8 wind (dx/dy is the drag direction).
 * @param {number} kind
 * @param {number} x
 * @param {number} y
 * @param {number} radius
 * @param {number} dx
 * @param {number} dy
 */
export function apply_tool(kind, x, y, radius, dx, dy) {
    wasm.apply_tool(kind, x, y, radius, dx, dy);
}

export function cycle_air() {
    wasm.cycle_air();
}

export function cycle_edge() {
    wasm.cycle_edge();
}

export function cycle_gravity() {
    wasm.cycle_gravity();
}

/**
 * @param {number} slot
 * @returns {boolean}
 */
export function despawn_fighter(slot) {
    const ret = wasm.despawn_fighter(slot);
    return ret !== 0;
}

/**
 * Despawn player `which`.
 * @param {number} which
 * @returns {boolean}
 */
export function despawn_stickman(which) {
    const ret = wasm.despawn_stickman(which);
    return ret !== 0;
}

/**
 * Number of particles of one PT id.
 * @param {number} t
 * @returns {number}
 */
export function element_count(t) {
    const ret = wasm.element_count(t);
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function fighter_count() {
    const ret = wasm.fighter_count();
    return ret >>> 0;
}

/**
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function get_cell_material(x, y) {
    const ret = wasm.get_cell_material(x, y);
    return ret;
}

/**
 * Air pressure at `(x, y)` in the reference's pressure units.
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function get_cell_pressure(x, y) {
    const ret = wasm.get_cell_pressure(x, y);
    return ret;
}

/**
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function get_cell_temperature(x, y) {
    const ret = wasm.get_cell_temperature(x, y);
    return ret;
}

/**
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function get_cell_vx(x, y) {
    const ret = wasm.get_cell_vx(x, y);
    return ret;
}

/**
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function get_cell_vy(x, y) {
    const ret = wasm.get_cell_vy(x, y);
    return ret;
}

/**
 * Wall id at `(x, y)` (0 = none).
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function get_wall(x, y) {
    const ret = wasm.get_wall(x, y);
    return ret;
}

/**
 * Initialize the simulation. `width`/`height` must be divisible by CELL (4).
 * @param {number} width
 * @param {number} height
 */
export function init(width, height) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        wasm.init(retptr, width, height);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeFromExternrefTable0(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Reason / summary of the most recent `load_blob` call. Empty before any
 * load. On success of a legacy import this includes how many particles were
 * migrated and which state could not be represented.
 * @returns {string}
 */
export function last_load_report() {
    let deferred1_0;
    let deferred1_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        wasm.last_load_report(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred1_0 = r0;
        deferred1_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export2(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Load a save. Accepts the native v2 format (`LMT2`) and the legacy v1
 * Cell-based format (`RMT\0`, optionally `ZLIB`-wrapped). The active world is
 * left untouched on any error. Legacy material ids are mapped through their
 * canonical TPT abbreviation, never reused numerically.
 * @param {Uint8Array | number[]} bytes
 * @returns {boolean}
 */
export function load_blob(bytes) {
    const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_export3);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.load_blob(ptr0, len0);
    return ret !== 0;
}

/**
 * Set a wall in the CELL containing `(x, y)`. `w == 0` erases the wall.
 * @param {number} x
 * @param {number} y
 * @param {number} w
 */
export function paint_wall(x, y, w) {
    wasm.paint_wall(x, y, w);
}

/**
 * Live particle count (alive slots, ordinary + energy).
 * @returns {number}
 */
export function particle_count() {
    const ret = wasm.particle_count();
    return ret >>> 0;
}

/**
 * Number of particles currently held in portal buffers.
 * @returns {number}
 */
export function portal_count() {
    const ret = wasm.portal_count();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function render_buffer_len() {
    const ret = wasm.render_buffer_len();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function render_buffer_ptr() {
    const ret = wasm.render_buffer_ptr();
    return ret >>> 0;
}

/**
 * @returns {boolean}
 */
export function reset() {
    const ret = wasm.reset();
    return ret !== 0;
}

/**
 * Serialize the current world (format v2). Empty vec if uninitialized.
 * @returns {Uint8Array}
 */
export function save_blob() {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        wasm.save_blob(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export2(r0, r1 * 1, 1);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {number} mode
 */
export function set_air_mode(mode) {
    wasm.set_air_mode(mode);
}

/**
 * @param {number} on
 */
export function set_ambient_heat(on) {
    wasm.set_ambient_heat(on);
}

/**
 * Paint a particle (PT id space). `material == 0` erases the pixel.
 * @param {number} x
 * @param {number} y
 * @param {number} material
 * @returns {boolean}
 */
export function set_cell(x, y, material) {
    const ret = wasm.set_cell(x, y, material);
    return ret !== 0;
}

/**
 * @param {number} mode
 */
export function set_convection_mode(mode) {
    wasm.set_convection_mode(mode);
}

/**
 * @param {number} mode
 */
export function set_edge_mode(mode) {
    wasm.set_edge_mode(mode);
}

/**
 * @param {number} slot
 * @param {number} comm
 */
export function set_fighter_input(slot, comm) {
    wasm.set_fighter_input(slot, comm);
}

/**
 * @param {number} mode
 */
export function set_gravity_mode(mode) {
    wasm.set_gravity_mode(mode);
}

/**
 * @param {number} on
 */
export function set_heat_sim(on) {
    wasm.set_heat_sim(on);
}

/**
 * @param {number} on
 */
export function set_newtonian(on) {
    wasm.set_newtonian(on);
}

/**
 * Set the element power spawned by the `SPAWN` command bit.
 * @param {number} which
 * @param {number} t
 */
export function set_stickman_element(which, t) {
    wasm.set_stickman_element(which, t);
}

/**
 * Set command bits: `1` left, `2` right, `4` jump, `8` spawn element.
 * @param {number} which
 * @param {number} comm
 */
export function set_stickman_input(which, comm) {
    wasm.set_stickman_input(which, comm);
}

/**
 * @param {number} mode
 */
export function set_view_mode(mode) {
    wasm.set_view_mode(mode);
}

/**
 * @returns {string}
 */
export function sim_status() {
    let deferred1_0;
    let deferred1_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        wasm.sim_status(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred1_0 = r0;
        deferred1_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export2(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Spawn a fighter. Returns its slot, or -1 when the registry is full.
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function spawn_fighter(x, y) {
    const ret = wasm.spawn_fighter(x, y);
    return ret;
}

/**
 * Spawn player `which` (0 = player 1, 1 = player 2) at a pixel.
 * @param {number} which
 * @param {number} x
 * @param {number} y
 * @returns {boolean}
 */
export function spawn_stickman(which, x, y) {
    const ret = wasm.spawn_stickman(which, x, y);
    return ret !== 0;
}

/**
 * @param {number} _dt
 */
export function step(_dt) {
    wasm.step(_dt);
}

/**
 * Advance the simulation without rebuilding the CPU render buffer. Used by
 * diagnostics/benchmarks to measure pure simulation cost separately from the
 * render-buffer build. The frontend does not call this; `step` remains the
 * normal entry point so rendering never goes stale during play.
 * @param {number} _dt
 */
export function step_sim_only(_dt) {
    wasm.step_sim_only(_dt);
}

/**
 * @param {number} which
 * @returns {boolean}
 */
export function stickman_alive(which) {
    const ret = wasm.stickman_alive(which);
    return ret !== 0;
}

/**
 * @returns {bigint}
 */
export function tick_count() {
    const ret = wasm.tick_count();
    return BigInt.asUintN(64, ret);
}

export function toggle_ambient() {
    wasm.toggle_ambient();
}

export function toggle_newtonian() {
    wasm.toggle_newtonian();
}

/**
 * Number of wireless channels active this frame.
 * @returns {number}
 */
export function wireless_active() {
    const ret = wasm.wireless_active();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function world_height() {
    const ret = wasm.world_height();
    return ret >>> 0;
}

/**
 * @returns {number}
 */
export function world_width() {
    const ret = wasm.world_width();
    return ret >>> 0;
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbindgen_generic_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            table.set(0, undefined);
            table.set(1024, undefined);
            table.set(1025, null);
            table.set(1026, true);
            table.set(1027, false);
        },
    };
    return {
        __proto__: null,
        "./labsmatter_wasm_bg.js": import0,
    };
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__wbindgen_export(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(bytes) {
    numBytesDecoded += bytes.length;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true });
        cachedTextDecoder.decode();
        numBytesDecoded = bytes.length;
    }
    return cachedTextDecoder.decode(bytes);
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

let __wbg_init_promise;
async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;
    if (__wbg_init_promise === undefined) {
        __wbg_init_promise = __wbg_init_once(module_or_path);
        __wbg_init_promise.catch(() => { __wbg_init_promise = undefined; });
    }
    return __wbg_init_promise;
}

async function __wbg_init_once(module_or_path) {

    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('labsmatter_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
