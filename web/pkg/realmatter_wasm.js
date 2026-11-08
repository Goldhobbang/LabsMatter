/**
 * `[material, temperature, vx, vy, flags, lifetime, pressure]`.
 * Empty / out of bounds is all zeros. There is no particle index to copy.
 * @param {number} x
 * @param {number} y
 * @returns {Float32Array}
 */
export function cell_state(x, y) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        wasm.cell_state(retptr, x, y);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayF32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export2(r0, r1 * 4, 4);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
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
 * Get a cell's material id (0 if out of bounds / uninitialized).
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function get_cell_material(x, y) {
    const ret = wasm.get_cell_material(x, y);
    return ret;
}

/**
 * Pressure in pascals (0 if out of bounds).
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function get_cell_pressure(x, y) {
    const ret = wasm.get_cell_pressure(x, y);
    return ret;
}

/**
 * Get a cell's temperature (Kelvin; 0.0 if out of bounds / uninitialized).
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function get_cell_temperature(x, y) {
    const ret = wasm.get_cell_temperature(x, y);
    return ret;
}

/**
 * Horizontal velocity, m/s (0 if out of bounds).
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function get_cell_vx(x, y) {
    const ret = wasm.get_cell_vx(x, y);
    return ret;
}

/**
 * Vertical velocity, m/s (0 if out of bounds). Positive is downward.
 * @param {number} x
 * @param {number} y
 * @returns {number}
 */
export function get_cell_vy(x, y) {
    const ret = wasm.get_cell_vy(x, y);
    return ret;
}

/**
 * Initialize the simulation. Called once from JS.
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
 * Last save/load failure. Empty after a successful load.
 * @returns {string}
 */
export function last_error() {
    let deferred1_0;
    let deferred1_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        wasm.last_error(retptr);
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
 * Load a persist-format blob into the world. Returns false on failure.
 * @param {Uint8Array | number[]} bytes
 * @returns {boolean}
 */
export function load_blob(bytes) {
    const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_export4);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.load_blob(ptr0, len0);
    return ret !== 0;
}

/**
 * Active WASM module. The page must not substitute another engine for this.
 * @returns {string}
 */
export function module_id() {
    let deferred1_0;
    let deferred1_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        wasm.module_id(retptr);
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
 * Occupied cells from the last full compose, adjusted by paints.
 * @returns {number}
 */
export function particle_count() {
    const ret = wasm.particle_count();
    return ret >>> 0;
}

/**
 * Length of the render buffer in bytes.
 * @returns {number}
 */
export function render_buffer_len() {
    const ret = wasm.render_buffer_len();
    return ret >>> 0;
}

/**
 * Pointer to the RGBA8 render buffer (row-major, y outer / x inner).
 * @returns {number}
 */
export function render_buffer_ptr() {
    const ret = wasm.render_buffer_ptr();
    return ret >>> 0;
}

/**
 * Last full framebuffer compose, in milliseconds. Not a browser frame time.
 * @returns {number}
 */
export function render_millis() {
    const ret = wasm.render_millis();
    return ret;
}

/**
 * Clear all cells and reset the tick counter. Returns false if uninitialized.
 * @returns {boolean}
 */
export function reset() {
    const ret = wasm.reset();
    return ret !== 0;
}

/**
 * Serialize the world to a persist-format blob (empty vec if uninitialized).
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
 * Air: 0 on, 1 off, 2 pressure only.
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
 * Set a cell's material. Returns true on success.
 *
 * New cells start at the material's characteristic temperature so thermal
 * interactions are observable: lava/fire/steam/smoke paint hot, ice paints
 * cold, everything else paints at ambient (293.15 K). Temperatures come
 * from the builtin material properties (boiling/melting points), not from
 * hardcoded magic numbers.
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
 * Restore or stamp a cell without resetting temperature, velocity, flags,
 * lifetime, or pressure. Spark stores its conductor id in `vx`; copying only
 * the material id would turn that spark into a fresh one. Non-finite numbers
 * are replaced so a bad stamp cannot poison the solver.
 * @param {number} x
 * @param {number} y
 * @param {number} material
 * @param {number} temperature
 * @param {number} vx
 * @param {number} vy
 * @param {number} flags
 * @param {number} lifetime
 * @param {number} pressure
 * @returns {boolean}
 */
export function set_cell_full(x, y, material, temperature, vx, vy, flags, lifetime, pressure) {
    const ret = wasm.set_cell_full(x, y, material, temperature, vx, vy, flags, lifetime, pressure);
    return ret !== 0;
}

/**
 * Edge: 0 void, 1 solid, 2 wrap.
 * @param {number} mode
 */
export function set_edge_mode(mode) {
    wasm.set_edge_mode(mode);
}

/**
 * Gravity: 0 down, 1 off, 2 toward center.
 * @param {number} mode
 */
export function set_gravity_mode(mode) {
    wasm.set_gravity_mode(mode);
}

/**
 * @param {number} on
 */
export function set_newtonian(on) {
    wasm.set_newtonian(on);
}

/**
 * 0 Normal, 1 Fancy, 2 Heat, 3 Pressure, 4 Velocity,
 * 5 Air velocity, 6 Air pressure, 7 Air heat, 8 Persistent, 9 Gravity.
 * @param {number} mode
 */
export function set_view_mode(mode) {
    wasm.set_view_mode(mode);
}

/**
 * Compact HUD string: gravity, air, edge, newton, ambient.
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
 * Advance simulation by one fixed step.
 *
 * Deterministic pipeline (approved tick phase order):
 * 1. Chemistry (registry reactions: combustion / oxidation / dissolution)
 * 2. Physics forces (gas pressure + diffusion + buoyancy, fluid viscosity +
 *    pressure + buoyancy, heat conduction, phase transitions, transfers)
 * 3. Movement (powder + liquid cellular automaton)
 * 4. Finalize (UPDATED flags, lifetimes, tick count) + render refresh.
 *
 * Operates directly on the world's cell slice; no duplicate state.
 * `_dt` is accepted for API compatibility; the fixed step comes from
 * `WorldConfig::fixed_dt` (1/60 s).
 * @param {number} _dt
 */
export function step(_dt) {
    wasm.step(_dt);
}

/**
 * Simulation tick counter (0 if uninitialized). Restored on load.
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
 * World height in cells (0 if uninitialized).
 * @returns {number}
 */
export function world_height() {
    const ret = wasm.world_height();
    return ret >>> 0;
}

/**
 * World width in cells (0 if uninitialized).
 * @returns {number}
 */
export function world_width() {
    const ret = wasm.world_width();
    return ret >>> 0;
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_is_undefined_be5b34249ff1fc15: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_throw_4c88d8bf264fd984: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_instanceof_Window_5cf453f5639aa1a2: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_now_043d245551b22f9c: function(arg0) {
            const ret = arg0.now();
            return ret;
        },
        __wbg_now_463ac581d2fc7ea3: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_performance_50e6a3f8dc80bbca: function(arg0) {
            const ret = arg0.performance;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_003ee833af9699fc: function() {
            const ret = typeof globalThis === 'undefined' ? undefined : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_e4293fb22cbe77da: function() {
            const ret = typeof global === 'undefined' ? undefined : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_53bb20457401453d: function() {
            const ret = typeof self === 'undefined' ? undefined : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_b5e0810bfecb7feb: function() {
            const ret = typeof window === 'undefined' ? undefined : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
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
        "./realmatter_wasm_bg.js": import0,
    };
}

function addToExternrefTable0(obj) {
    const idx = wasm.__wbindgen_export();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
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

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
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

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__wbindgen_export3(idx);
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
    cachedFloat32ArrayMemory0 = null;
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
        module_or_path = new URL('realmatter_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
