#include "../quickjs/quickjs.h"

// These are static inline functions in quickjs.h so bindgen does not pick 
// them up.
// We use define simple wrapper functions to make them available to bindgen,
// and therefore make them usable from Rust.

int JS_ValueGetTag_real(JSValue v) {
    return JS_VALUE_GET_TAG(v);
}

void JS_FreeValue_real(JSContext *ctx, JSValue v) {
    JS_FreeValue(ctx, v);
}

void JS_FreeValueRT_real(JSRuntime *rt, JSValue v) {
    return JS_FreeValueRT(rt, v);
}

JSValue JS_DupValue_real(JSContext *ctx, JSValue v) {
    return JS_DupValue(ctx, v);
}

JSValue JS_DupValueRT_real(JSRuntime *rt, JSValue v) {
    return JS_DupValueRT(rt, v);
}

JSValue JS_NewFloat64_real(JSContext *ctx, double d) {
    return JS_NewFloat64(ctx, d);
}

JSValue JS_NewInt32_real(JSContext *ctx, int32_t val) {
    return JS_NewInt32(ctx, val);
}

JSValue JS_NewInt64_real(JSContext *ctx, int64_t val) {
    return JS_NewInt64(ctx, val);
}

JSValue JS_NewUint32_real(JSContext *ctx, uint32_t val) {
    return JS_NewUint32(ctx, val);
}

JSValue JS_NewCatchOffset_real(JSContext *ctx, int32_t val) {
    return JS_NewCatchOffset(ctx, val);
}

JSValue JS_NewBool_real(JSContext *ctx, JS_BOOL val) {
    return JS_NewBool(ctx, val) ;
}

JS_BOOL JS_VALUE_IS_NAN_real(JSValue v) {
    return JS_VALUE_IS_NAN(v);
}

double JS_VALUE_GET_FLOAT64_real(JSValue v) {
    return JS_VALUE_GET_FLOAT64(v);
}

int JS_VALUE_GET_NORM_TAG_real(JSValue v) {
    return JS_VALUE_GET_NORM_TAG(v);
}

JS_BOOL JS_IsNumber_real(JSValueConst v) {
    return JS_IsNumber(v);
}

JS_BOOL JS_IsBigInt_real(JSContext *ctx, JSValueConst v) {
    return JS_IsBigInt(ctx, v);
}

JS_BOOL JS_IsBigFloat_real(JSValueConst v) {
    return JS_IsBigFloat(v);
}

JS_BOOL JS_IsBigDecimal_real(JSValueConst v) {
    return JS_IsBigDecimal(v);
}

JS_BOOL JS_IsBool_real(JSValueConst v) {
    return JS_IsBool(v);
}

JS_BOOL JS_IsNull_real(JSValueConst v) {
    return JS_IsNull(v);
}

JS_BOOL JS_IsUndefined_real(JSValueConst v) {
    return JS_IsUndefined(v);
}

JS_BOOL JS_IsException_real(JSValueConst v) {
    return JS_IsException(v);
}

JS_BOOL JS_IsUninitialized_real(JSValueConst v) {
    return JS_IsUninitialized(v);
}

JS_BOOL JS_IsString_real(JSValueConst v) {
    return JS_IsString(v);
}

JS_BOOL JS_IsSymbol_real(JSValueConst v) {
    return JS_IsSymbol(v);
}

JS_BOOL JS_IsObject_real(JSValueConst v) {
    return JS_IsObject(v);
}

int JS_ToUint32_real(JSContext *ctx, uint32_t *pres, JSValueConst val) {
    return JS_ToUint32(ctx, pres, val);
}

const char *JS_ToCStringLen_real(JSContext *ctx, size_t *plen, JSValueConst val1) {
    return JS_ToCStringLen(ctx, plen, val1);
}

const char *JS_ToCString_real(JSContext *ctx, JSValueConst val1) {
    return JS_ToCString(ctx, val1);
}

int JS_SetProperty_real(JSContext *ctx, JSValueConst this_obj, JSAtom prop, JSValue val) {
    return JS_SetProperty(ctx, this_obj, prop, val);
}

JSValue JS_NewCFunction_real(JSContext *ctx, JSCFunction *func, const char *name,int length) {
    return JS_NewCFunction(ctx, func, name, length);
}

JSValue JS_NewCFunctionMagic_real(JSContext *ctx, JSCFunctionMagic *func, const char *name, int length, JSCFunctionEnum cproto, int magic) {
    return JS_NewCFunctionMagic(ctx, func, name, length, cproto, magic);
}
