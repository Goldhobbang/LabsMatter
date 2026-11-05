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
 * Load a persist-format blob into the world. Returns false on failure.
 * @param {Uint8Array | number[]} bytes
 * @returns {boolean}
 */
export function load_blob(bytes) {
    const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.load_blob(ptr0, len0);
    return ret !== 0;
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
        wasm.__wbindgen_export3(r0, r1 * 1, 1);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
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
 * 0 material, 1 fancy (heat tint), 2 heat, 3 pressure, 4 velocity.
 * Colors come from the live cell fields, not from invented values.
 * @param {number} mode
 */
export function set_view_mode(mode) {
    wasm.set_view_mode(mode);
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
