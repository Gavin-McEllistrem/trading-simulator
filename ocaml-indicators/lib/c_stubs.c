/* C FFI stubs for OCaml indicator library */

#include <caml/mlvalues.h>
#include <caml/memory.h>
#include <caml/alloc.h>
#include <caml/callback.h>
#include <caml/fail.h>
#include <string.h>

/* Helper to convert C double array to OCaml float array */
static value doubles_to_ocaml_array(const double *data, int len) {
    CAMLparam0();
    CAMLlocal1(ml_array);

    ml_array = caml_alloc(len * Double_wosize, Double_array_tag);
    for (int i = 0; i < len; i++) {
        Store_double_field(ml_array, i, data[i]);
    }

    CAMLreturn(ml_array);
}

/* Helper to convert OCaml float array to C double array */
static void ocaml_array_to_doubles(value ml_array, double *out, int len) {
    for (int i = 0; i < len && i < Wosize_val(ml_array) / Double_wosize; i++) {
        out[i] = Double_field(ml_array, i);
    }
}

/* SMA C interface */
CAMLprim value caml_sma(value data_val, value period_val) {
    CAMLparam2(data_val, period_val);
    CAMLlocal1(result);

    static const value *sma_closure = NULL;
    if (sma_closure == NULL) {
        sma_closure = caml_named_value("sma_ffi");
        if (sma_closure == NULL) {
            caml_failwith("SMA function not registered");
        }
    }

    result = caml_callback2(*sma_closure, data_val, period_val);
    CAMLreturn(result);
}

/* EMA C interface */
CAMLprim value caml_ema(value data_val, value period_val) {
    CAMLparam2(data_val, period_val);
    CAMLlocal1(result);

    static const value *ema_closure = NULL;
    if (ema_closure == NULL) {
        ema_closure = caml_named_value("ema_ffi");
        if (ema_closure == NULL) {
            caml_failwith("EMA function not registered");
        }
    }

    result = caml_callback2(*ema_closure, data_val, period_val);
    CAMLreturn(result);
}

/* RSI C interface */
CAMLprim value caml_rsi(value data_val, value period_val) {
    CAMLparam2(data_val, period_val);
    CAMLlocal1(result);

    static const value *rsi_closure = NULL;
    if (rsi_closure == NULL) {
        rsi_closure = caml_named_value("rsi_ffi");
        if (rsi_closure == NULL) {
            caml_failwith("RSI function not registered");
        }
    }

    result = caml_callback2(*rsi_closure, data_val, period_val);
    CAMLreturn(result);
}

/* MACD C interface */
CAMLprim value caml_macd(value data_val, value fast_val, value slow_val, value signal_val) {
    CAMLparam4(data_val, fast_val, slow_val, signal_val);
    CAMLlocal1(result);

    static const value *macd_closure = NULL;
    if (macd_closure == NULL) {
        macd_closure = caml_named_value("macd_ffi");
        if (macd_closure == NULL) {
            caml_failwith("MACD function not registered");
        }
    }

    result = caml_callback3(*macd_closure, data_val,
                           caml_callback2(*macd_closure, fast_val, slow_val),
                           signal_val);
    CAMLreturn(result);
}

/* Bollinger Bands C interface */
CAMLprim value caml_bollinger_bands(value data_val, value period_val, value std_val) {
    CAMLparam3(data_val, period_val, std_val);
    CAMLlocal1(result);

    static const value *bb_closure = NULL;
    if (bb_closure == NULL) {
        bb_closure = caml_named_value("bollinger_bands_ffi");
        if (bb_closure == NULL) {
            caml_failwith("Bollinger Bands function not registered");
        }
    }

    result = caml_callback3(*bb_closure, data_val, period_val, std_val);
    CAMLreturn(result);
}

/* Initialize the OCaml runtime - MUST be called before any other functions */
CAMLprim void indicators_init(void) {
    char *argv[] = { "indicators_ffi", NULL };
    caml_startup(argv);
}
