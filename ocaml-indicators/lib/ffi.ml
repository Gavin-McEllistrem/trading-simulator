(** FFI wrapper functions for C interop *)

open Indicators

(** Convert C double array to OCaml float array *)
let array_of_ptr (ptr : float array) : float array = ptr

(** Export SMA function for C FFI *)
let sma_ffi data period =
  try
    let result = sma data period in
    result
  with Invalid_argument msg ->
    Printf.eprintf "SMA error: %s\n" msg;
    [||]

(** Export EMA function for C FFI *)
let ema_ffi data period =
  try
    let result = ema data period in
    result
  with Invalid_argument msg ->
    Printf.eprintf "EMA error: %s\n" msg;
    [||]

(** Export RSI function for C FFI *)
let rsi_ffi data period =
  try
    let result = rsi data period in
    result
  with Invalid_argument msg ->
    Printf.eprintf "RSI error: %s\n" msg;
    [||]

(** Export MACD function for C FFI
    Returns concatenated array: [macd_line; signal_line; histogram] *)
let macd_ffi data fast slow signal =
  try
    let (macd_line, signal_line, histogram) = macd data fast slow signal in
    Array.concat [macd_line; signal_line; histogram]
  with Invalid_argument msg ->
    Printf.eprintf "MACD error: %s\n" msg;
    [||]

(** Export Bollinger Bands for C FFI
    Returns concatenated array: [upper; middle; lower] *)
let bollinger_bands_ffi data period num_std =
  try
    let (upper, middle, lower) = bollinger_bands data period num_std in
    Array.concat [upper; middle; lower]
  with Invalid_argument msg ->
    Printf.eprintf "Bollinger Bands error: %s\n" msg;
    [||]

(* Register callbacks with C *)
let () =
  Callback.register "sma_ffi" sma_ffi;
  Callback.register "ema_ffi" ema_ffi;
  Callback.register "rsi_ffi" rsi_ffi;
  Callback.register "macd_ffi" macd_ffi;
  Callback.register "bollinger_bands_ffi" bollinger_bands_ffi
