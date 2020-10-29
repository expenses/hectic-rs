import * as wasm from './hectic-rs_bg.wasm';

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

const lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachegetUint8Memory0 = null;
function getUint8Memory0() {
    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachegetFloat64Memory0 = null;
function getFloat64Memory0() {
    if (cachegetFloat64Memory0 === null || cachegetFloat64Memory0.buffer !== wasm.memory.buffer) {
        cachegetFloat64Memory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachegetFloat64Memory0;
}

let cachegetInt32Memory0 = null;
function getInt32Memory0() {
    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== wasm.memory.buffer) {
        cachegetInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachegetInt32Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

let WASM_VECTOR_LEN = 0;

const lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;

let cachedTextEncoder = new lTextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length);
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len);

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3);
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);

            } else {
                state.a = a;
            }
        }
    };
    real.original = state;

    return real;
}
function __wbg_adapter_22(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h071d281e80e716c9(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_25(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h23862a73b5519379(arg0, arg1);
}

function __wbg_adapter_28(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h071d281e80e716c9(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_31(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h071d281e80e716c9(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_34(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h071d281e80e716c9(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_37(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h071d281e80e716c9(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_40(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h071d281e80e716c9(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_43(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h071d281e80e716c9(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_46(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h0b26e319ec05688f(arg0, arg1, addHeapObject(arg2));
}

function handleError(f) {
    return function () {
        try {
            return f.apply(this, arguments);

        } catch (e) {
            wasm.__wbindgen_exn_store(addHeapObject(e));
        }
    };
}

function getArrayU8FromWasm0(ptr, len) {
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachegetUint32Memory0 = null;
function getUint32Memory0() {
    if (cachegetUint32Memory0 === null || cachegetUint32Memory0.buffer !== wasm.memory.buffer) {
        cachegetUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachegetUint32Memory0;
}

function getArrayU32FromWasm0(ptr, len) {
    return getUint32Memory0().subarray(ptr / 4, ptr / 4 + len);
}

export const __wbindgen_object_drop_ref = function(arg0) {
    takeObject(arg0);
};

export const __wbindgen_cb_drop = function(arg0) {
    const obj = takeObject(arg0).original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    var ret = false;
    return ret;
};

export const __wbindgen_object_clone_ref = function(arg0) {
    var ret = getObject(arg0);
    return addHeapObject(ret);
};

export const __wbindgen_string_new = function(arg0, arg1) {
    var ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
};

export const __wbg_new_59cb74e423758ede = function() {
    var ret = new Error();
    return addHeapObject(ret);
};

export const __wbg_stack_558ba5917b466edd = function(arg0, arg1) {
    var ret = getObject(arg1).stack;
    var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export const __wbg_error_4bb6c2a97407129a = function(arg0, arg1) {
    try {
        console.error(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_free(arg0, arg1);
    }
};

export const __wbg_self_1b7a39e3a92c949c = handleError(function() {
    var ret = self.self;
    return addHeapObject(ret);
});

export const __wbg_require_604837428532a733 = function(arg0, arg1) {
    var ret = require(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export const __wbg_crypto_968f1772287e2df0 = function(arg0) {
    var ret = getObject(arg0).crypto;
    return addHeapObject(ret);
};

export const __wbindgen_is_undefined = function(arg0) {
    var ret = getObject(arg0) === undefined;
    return ret;
};

export const __wbg_getRandomValues_a3d34b4fee3c2869 = function(arg0) {
    var ret = getObject(arg0).getRandomValues;
    return addHeapObject(ret);
};

export const __wbg_getRandomValues_f5e14ab7ac8e995d = function(arg0, arg1, arg2) {
    getObject(arg0).getRandomValues(getArrayU8FromWasm0(arg1, arg2));
};

export const __wbg_randomFillSync_d5bd2d655fdf256a = function(arg0, arg1, arg2) {
    getObject(arg0).randomFillSync(getArrayU8FromWasm0(arg1, arg2));
};

export const __wbindgen_number_new = function(arg0) {
    var ret = arg0;
    return addHeapObject(ret);
};

export const __wbg_instanceof_Window_e8f84259147dce74 = function(arg0) {
    var ret = getObject(arg0) instanceof Window;
    return ret;
};

export const __wbg_document_d3b6d86af1c5d199 = function(arg0) {
    var ret = getObject(arg0).document;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export const __wbg_navigator_06614ec1a7c6bb66 = function(arg0) {
    var ret = getObject(arg0).navigator;
    return addHeapObject(ret);
};

export const __wbg_innerWidth_2a084ee2fb8c0457 = handleError(function(arg0) {
    var ret = getObject(arg0).innerWidth;
    return addHeapObject(ret);
});

export const __wbg_innerHeight_4676de3f9d6f79be = handleError(function(arg0) {
    var ret = getObject(arg0).innerHeight;
    return addHeapObject(ret);
});

export const __wbg_devicePixelRatio_8e0818d196b8e065 = function(arg0) {
    var ret = getObject(arg0).devicePixelRatio;
    return ret;
};

export const __wbg_cancelAnimationFrame_396f71da29fb2b46 = handleError(function(arg0, arg1) {
    getObject(arg0).cancelAnimationFrame(arg1);
});

export const __wbg_matchMedia_76dc27be2f13026a = handleError(function(arg0, arg1, arg2) {
    var ret = getObject(arg0).matchMedia(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
});

export const __wbg_requestAnimationFrame_e5d576010b9bc3a3 = handleError(function(arg0, arg1) {
    var ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
    return ret;
});

export const __wbg_clearTimeout_0bc6122edf81bfe9 = function(arg0, arg1) {
    getObject(arg0).clearTimeout(arg1);
};

export const __wbg_setTimeout_d0eb4368d101bd72 = handleError(function(arg0, arg1, arg2) {
    var ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
    return ret;
});

export const __wbg_body_61c142aa6eae691f = function(arg0) {
    var ret = getObject(arg0).body;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export const __wbg_fullscreenElement_397f3b426943a1dd = function(arg0) {
    var ret = getObject(arg0).fullscreenElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export const __wbg_createElement_d00b8e24838e36e1 = handleError(function(arg0, arg1, arg2) {
    var ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
    return addHeapObject(ret);
});

export const __wbg_querySelectorAll_4af162be59c08fa0 = handleError(function(arg0, arg1, arg2) {
    var ret = getObject(arg0).querySelectorAll(getStringFromWasm0(arg1, arg2));
    return addHeapObject(ret);
});

export const __wbg_cancelBubble_7a3707693157f9fd = function(arg0) {
    var ret = getObject(arg0).cancelBubble;
    return ret;
};

export const __wbg_stopPropagation_b607ef55ec1122f3 = function(arg0) {
    getObject(arg0).stopPropagation();
};

export const __wbg_submit_ecd6d663ec2625f4 = function(arg0, arg1) {
    getObject(arg0).submit(getObject(arg1));
};

export const __wbg_getCurrentTexture_ba294c1e75ddd4f0 = function(arg0) {
    var ret = getObject(arg0).getCurrentTexture();
    return addHeapObject(ret);
};

export const __wbg_width_175e0a733f9f4219 = function(arg0) {
    var ret = getObject(arg0).width;
    return ret;
};

export const __wbg_setwidth_8d33dd91eeeee87d = function(arg0, arg1) {
    getObject(arg0).width = arg1 >>> 0;
};

export const __wbg_height_d91cbd8f64ea6e32 = function(arg0) {
    var ret = getObject(arg0).height;
    return ret;
};

export const __wbg_setheight_757ff0f25240fd75 = function(arg0, arg1) {
    getObject(arg0).height = arg1 >>> 0;
};

export const __wbg_getContext_59043a63a2f9266b = handleError(function(arg0, arg1, arg2) {
    var ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
});

export const __wbg_get_6d6e3c7bc98181de = function(arg0, arg1) {
    var ret = getObject(arg0)[arg1 >>> 0];
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export const __wbg_requestDevice_680fb1b3bed315eb = function(arg0, arg1) {
    var ret = getObject(arg0).requestDevice(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_configureSwapChain_2185ccd0582af588 = function(arg0, arg1) {
    var ret = getObject(arg0).configureSwapChain(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_label_be626939bedaf56d = function(arg0, arg1) {
    var ret = getObject(arg1).label;
    var ptr0 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export const __wbg_beginRenderPass_ba6b7660837435a7 = function(arg0, arg1) {
    var ret = getObject(arg0).beginRenderPass(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_copyBufferToBuffer_d744e3e841826fd9 = function(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).copyBufferToBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
};

export const __wbg_copyBufferToTexture_9aae15799f7c5470 = function(arg0, arg1, arg2, arg3) {
    getObject(arg0).copyBufferToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
};

export const __wbg_finish_c3fcf7fe4644dff6 = function(arg0, arg1) {
    var ret = getObject(arg0).finish(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_addListener_2e81f0c1e9edfa62 = handleError(function(arg0, arg1) {
    getObject(arg0).addListener(getObject(arg1));
});

export const __wbg_gpu_ed4108c374b54ada = function(arg0) {
    var ret = getObject(arg0).gpu;
    return addHeapObject(ret);
};

export const __wbg_appendChild_8658f795c44d1316 = handleError(function(arg0, arg1) {
    var ret = getObject(arg0).appendChild(getObject(arg1));
    return addHeapObject(ret);
});

export const __wbg_endPass_d1f8f3b8680f19eb = function(arg0) {
    getObject(arg0).endPass();
};

export const __wbg_setScissorRect_2b8789ced7c4ee97 = function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setScissorRect(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
};

export const __wbg_setBindGroup_3af09880fae13c89 = function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
};

export const __wbg_draw_b623bc97e6b43830 = function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
};

export const __wbg_setPipeline_56c5528385a907e4 = function(arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
};

export const __wbg_setVertexBuffer_29bc814118d56970 = function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
};

export const __wbg_requestFullscreen_3f3a3f14194196f2 = handleError(function(arg0) {
    getObject(arg0).requestFullscreen();
});

export const __wbg_setAttribute_156f15ecfed9f628 = handleError(function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
});

export const __wbg_remove_753943fab80b89c7 = function(arg0) {
    getObject(arg0).remove();
};

export const __wbg_debug_ef2b78738889619f = function(arg0) {
    console.debug(getObject(arg0));
};

export const __wbg_error_7dcc755846c00ef7 = function(arg0) {
    console.error(getObject(arg0));
};

export const __wbg_info_43f70b84e943346e = function(arg0) {
    console.info(getObject(arg0));
};

export const __wbg_log_61ea781bd002cc41 = function(arg0) {
    console.log(getObject(arg0));
};

export const __wbg_warn_502e53bc79de489a = function(arg0) {
    console.warn(getObject(arg0));
};

export const __wbg_style_ae2bb40204a83a34 = function(arg0) {
    var ret = getObject(arg0).style;
    return addHeapObject(ret);
};

export const __wbg_setProperty_4a05a7c81066031f = handleError(function(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
});

export const __wbg_addEventListener_116c561435e7160d = handleError(function(arg0, arg1, arg2, arg3) {
    getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
});

export const __wbg_requestAdapter_73fad3f24e7a8e82 = function(arg0, arg1) {
    var ret = getObject(arg0).requestAdapter(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_offsetX_08c9c32119cefae0 = function(arg0) {
    var ret = getObject(arg0).offsetX;
    return ret;
};

export const __wbg_offsetY_d2d82e37cd77b9e5 = function(arg0) {
    var ret = getObject(arg0).offsetY;
    return ret;
};

export const __wbg_ctrlKey_da0b27f443c75e18 = function(arg0) {
    var ret = getObject(arg0).ctrlKey;
    return ret;
};

export const __wbg_shiftKey_5faa6c16a9599f01 = function(arg0) {
    var ret = getObject(arg0).shiftKey;
    return ret;
};

export const __wbg_altKey_8f65ec92db7e582c = function(arg0) {
    var ret = getObject(arg0).altKey;
    return ret;
};

export const __wbg_metaKey_0b52f758eccc8995 = function(arg0) {
    var ret = getObject(arg0).metaKey;
    return ret;
};

export const __wbg_button_69638b9dba7a0f91 = function(arg0) {
    var ret = getObject(arg0).button;
    return ret;
};

export const __wbg_createView_ba32e0297c4cb2b4 = function(arg0) {
    var ret = getObject(arg0).createView();
    return addHeapObject(ret);
};

export const __wbg_createView_4714d4c828561e98 = function(arg0, arg1) {
    var ret = getObject(arg0).createView(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_pointerId_ae033d6fc8ddb366 = function(arg0) {
    var ret = getObject(arg0).pointerId;
    return ret;
};

export const __wbg_deltaX_d3527e144ad7b020 = function(arg0) {
    var ret = getObject(arg0).deltaX;
    return ret;
};

export const __wbg_deltaY_382e72a682f18515 = function(arg0) {
    var ret = getObject(arg0).deltaY;
    return ret;
};

export const __wbg_deltaMode_afd49f429e5a6a7f = function(arg0) {
    var ret = getObject(arg0).deltaMode;
    return ret;
};

export const __wbg_now_acfa6ea53a7be2c2 = function(arg0) {
    var ret = getObject(arg0).now();
    return ret;
};

export const __wbg_defaultQueue_35ea47f77bf3091e = function(arg0) {
    var ret = getObject(arg0).defaultQueue;
    return addHeapObject(ret);
};

export const __wbg_createBindGroup_b63679e2f7ac5e97 = function(arg0, arg1) {
    var ret = getObject(arg0).createBindGroup(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_createBindGroupLayout_04a7875976f74c1e = function(arg0, arg1) {
    var ret = getObject(arg0).createBindGroupLayout(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_createBuffer_0016a580281dc0ef = function(arg0, arg1) {
    var ret = getObject(arg0).createBuffer(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_createCommandEncoder_7c02c6fe432ccc69 = function(arg0, arg1) {
    var ret = getObject(arg0).createCommandEncoder(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_createPipelineLayout_ee91987f228529cd = function(arg0, arg1) {
    var ret = getObject(arg0).createPipelineLayout(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_createRenderPipeline_9cd0eadbe87562a0 = function(arg0, arg1) {
    var ret = getObject(arg0).createRenderPipeline(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_createSampler_e26a3c623066229e = function(arg0, arg1) {
    var ret = getObject(arg0).createSampler(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_createShaderModule_97b73fbc79bb3ab0 = function(arg0, arg1) {
    var ret = getObject(arg0).createShaderModule(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_createTexture_4e44e19745b67c23 = function(arg0, arg1) {
    var ret = getObject(arg0).createTexture(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_matches_8e307f130803b2ff = function(arg0) {
    var ret = getObject(arg0).matches;
    return ret;
};

export const __wbg_charCode_43f861f4a06baf46 = function(arg0) {
    var ret = getObject(arg0).charCode;
    return ret;
};

export const __wbg_keyCode_d74097d530e093a8 = function(arg0) {
    var ret = getObject(arg0).keyCode;
    return ret;
};

export const __wbg_altKey_501212f36ae811a4 = function(arg0) {
    var ret = getObject(arg0).altKey;
    return ret;
};

export const __wbg_ctrlKey_e2778fe941bb5156 = function(arg0) {
    var ret = getObject(arg0).ctrlKey;
    return ret;
};

export const __wbg_shiftKey_072ed91b9a400bcb = function(arg0) {
    var ret = getObject(arg0).shiftKey;
    return ret;
};

export const __wbg_metaKey_ab904088bd961450 = function(arg0) {
    var ret = getObject(arg0).metaKey;
    return ret;
};

export const __wbg_key_0b3d2c7a78af4571 = function(arg0, arg1) {
    var ret = getObject(arg1).key;
    var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export const __wbg_code_59e0af7de7519251 = function(arg0, arg1) {
    var ret = getObject(arg1).code;
    var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export const __wbg_get_2e96a823c1c5a5bd = handleError(function(arg0, arg1) {
    var ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
});

export const __wbg_call_e9f0ce4da840ab94 = handleError(function(arg0, arg1) {
    var ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
});

export const __wbg_new_17534eac4df3cd22 = function() {
    var ret = new Array();
    return addHeapObject(ret);
};

export const __wbg_push_7114ccbf1c58e41f = function(arg0, arg1) {
    var ret = getObject(arg0).push(getObject(arg1));
    return ret;
};

export const __wbg_newnoargs_e2fdfe2af14a2323 = function(arg0, arg1) {
    var ret = new Function(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export const __wbg_is_a2bc492e20d950cf = function(arg0, arg1) {
    var ret = Object.is(getObject(arg0), getObject(arg1));
    return ret;
};

export const __wbg_new_8172f4fed77fdb7c = function() {
    var ret = new Object();
    return addHeapObject(ret);
};

export const __wbg_resolve_4df26938859b92e3 = function(arg0) {
    var ret = Promise.resolve(getObject(arg0));
    return addHeapObject(ret);
};

export const __wbg_then_ffb6e71f7a6735ad = function(arg0, arg1) {
    var ret = getObject(arg0).then(getObject(arg1));
    return addHeapObject(ret);
};

export const __wbg_then_021fcdc7f0350b58 = function(arg0, arg1, arg2) {
    var ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
};

export const __wbg_self_179e8c2a5a4c73a3 = handleError(function() {
    var ret = self.self;
    return addHeapObject(ret);
});

export const __wbg_window_492cfe63a6e41dfa = handleError(function() {
    var ret = window.window;
    return addHeapObject(ret);
});

export const __wbg_globalThis_8ebfea75c2dd63ee = handleError(function() {
    var ret = globalThis.globalThis;
    return addHeapObject(ret);
});

export const __wbg_global_62ea2619f58bf94d = handleError(function() {
    var ret = global.global;
    return addHeapObject(ret);
});

export const __wbg_buffer_88f603259d7a7b82 = function(arg0) {
    var ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

export const __wbg_newwithbyteoffsetandlength_5ba4b4465eeaa8d3 = function(arg0, arg1, arg2) {
    var ret = new Uint32Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export const __wbg_new_2d3653c61bf7d9f5 = function(arg0) {
    var ret = new Uint32Array(getObject(arg0));
    return addHeapObject(ret);
};

export const __wbg_set_afe54b1eeb1aa77c = handleError(function(arg0, arg1, arg2) {
    var ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
    return ret;
});

export const __wbindgen_number_get = function(arg0, arg1) {
    const obj = getObject(arg1);
    var ret = typeof(obj) === 'number' ? obj : undefined;
    getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
};

export const __wbindgen_debug_string = function(arg0, arg1) {
    var ret = debugString(getObject(arg1));
    var ptr0 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len0;
    getInt32Memory0()[arg0 / 4 + 0] = ptr0;
};

export const __wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

export const __wbindgen_memory = function() {
    var ret = wasm.memory;
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper1718 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 476, __wbg_adapter_46);
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper408 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 96, __wbg_adapter_40);
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper410 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 96, __wbg_adapter_34);
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper418 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 96, __wbg_adapter_43);
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper416 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 96, __wbg_adapter_25);
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper412 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 96, __wbg_adapter_37);
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper422 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 96, __wbg_adapter_28);
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper414 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 96, __wbg_adapter_31);
    return addHeapObject(ret);
};

export const __wbindgen_closure_wrapper420 = function(arg0, arg1, arg2) {
    var ret = makeMutClosure(arg0, arg1, 96, __wbg_adapter_22);
    return addHeapObject(ret);
};

