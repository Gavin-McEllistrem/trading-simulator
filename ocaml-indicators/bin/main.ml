(** CLI for technical indicators - accepts JSON input, outputs JSON results *)

open Indicators
open Yojson.Safe

(** Parse float array from JSON *)
let parse_float_array json =
  match json with
  | `List items ->
      Array.of_list (List.map (function
        | `Float f -> f
        | `Int i -> float_of_int i
        | _ -> failwith "Expected numeric array"
      ) items)
  | _ -> failwith "Expected array"

(** Convert float array to JSON *)
let float_array_to_json arr =
  `List (Array.to_list (Array.map (fun f -> `Float f) arr))

(** Process SMA request *)
let process_sma data period =
  let result = sma data period in
  `Assoc [
    ("indicator", `String "sma");
    ("period", `Int period);
    ("values", float_array_to_json result);
  ]

(** Process EMA request *)
let process_ema data period =
  let result = ema data period in
  `Assoc [
    ("indicator", `String "ema");
    ("period", `Int period);
    ("values", float_array_to_json result);
  ]

(** Process RSI request *)
let process_rsi data period =
  let result = rsi data period in
  `Assoc [
    ("indicator", `String "rsi");
    ("period", `Int period);
    ("values", float_array_to_json result);
  ]

(** Process MACD request *)
let process_macd data fast slow signal =
  let (macd_line, signal_line, histogram) = macd data fast slow signal in
  `Assoc [
    ("indicator", `String "macd");
    ("fast_period", `Int fast);
    ("slow_period", `Int slow);
    ("signal_period", `Int signal);
    ("macd_line", float_array_to_json macd_line);
    ("signal_line", float_array_to_json signal_line);
    ("histogram", float_array_to_json histogram);
  ]

(** Process Bollinger Bands request *)
let process_bollinger_bands data period num_std =
  let (upper, middle, lower) = bollinger_bands data period num_std in
  `Assoc [
    ("indicator", `String "bollinger_bands");
    ("period", `Int period);
    ("num_std_dev", `Float num_std);
    ("upper", float_array_to_json upper);
    ("middle", float_array_to_json middle);
    ("lower", float_array_to_json lower);
  ]

(** Main request handler *)
let handle_request json =
  try
    match json with
    | `Assoc fields ->
        let indicator = List.assoc "indicator" fields in
        let data_json = List.assoc "data" fields in
        let data = parse_float_array data_json in

        begin match indicator with
        | `String "sma" ->
            let period = match List.assoc "period" fields with
              | `Int p -> p
              | _ -> failwith "period must be integer"
            in
            process_sma data period

        | `String "ema" ->
            let period = match List.assoc "period" fields with
              | `Int p -> p
              | _ -> failwith "period must be integer"
            in
            process_ema data period

        | `String "rsi" ->
            let period = match List.assoc "period" fields with
              | `Int p -> p
              | _ -> failwith "period must be integer"
            in
            process_rsi data period

        | `String "macd" ->
            let fast = match List.assoc "fast_period" fields with
              | `Int p -> p
              | _ -> failwith "fast_period must be integer"
            in
            let slow = match List.assoc "slow_period" fields with
              | `Int p -> p
              | _ -> failwith "slow_period must be integer"
            in
            let signal = match List.assoc "signal_period" fields with
              | `Int p -> p
              | _ -> failwith "signal_period must be integer"
            in
            process_macd data fast slow signal

        | `String "bollinger_bands" ->
            let period = match List.assoc "period" fields with
              | `Int p -> p
              | _ -> failwith "period must be integer"
            in
            let num_std = match List.assoc "num_std_dev" fields with
              | `Float f -> f
              | `Int i -> float_of_int i
              | _ -> failwith "num_std_dev must be numeric"
            in
            process_bollinger_bands data period num_std

        | `String name ->
            `Assoc [
              ("error", `String (Printf.sprintf "Unknown indicator: %s" name));
            ]

        | _ -> `Assoc [("error", `String "indicator must be string")]
        end

    | _ -> `Assoc [("error", `String "Request must be JSON object")]

  with
  | Not_found -> `Assoc [("error", `String "Missing required field")]
  | Failure msg -> `Assoc [("error", `String msg)]
  | Invalid_argument msg -> `Assoc [("error", `String msg)]
  | exn -> `Assoc [("error", `String (Printexc.to_string exn))]

(** Interactive mode - read JSON from stdin line by line *)
let interactive_mode () =
  try
    while true do
      let line = read_line () in
      if String.trim line = "" then ()
      else if String.trim line = "exit" then raise Exit
      else
        try
          let request = from_string line in
          let response = handle_request request in
          print_endline (to_string response);
          flush stdout
        with
        | Yojson.Json_error msg ->
            let err = `Assoc [("error", `String ("JSON parse error: " ^ msg))] in
            print_endline (to_string err);
            flush stdout
    done
  with
  | End_of_file -> ()
  | Exit -> ()

(** Single request mode - read one JSON from stdin *)
let single_mode () =
  try
    let request = from_channel stdin in
    let response = handle_request request in
    print_endline (to_string response)
  with
  | Yojson.Json_error msg ->
      let err = `Assoc [("error", `String ("JSON parse error: " ^ msg))] in
      print_endline (to_string err);
      exit 1

(** Main entry point *)
let () =
  let mode = if Array.length Sys.argv > 1 && Sys.argv.(1) = "--interactive"
             then `Interactive
             else `Single
  in
  match mode with
  | `Interactive ->
      Printf.eprintf "Indicators CLI - Interactive Mode\n";
      Printf.eprintf "Enter JSON requests (one per line), 'exit' to quit\n";
      flush stderr;
      interactive_mode ()
  | `Single ->
      single_mode ()
