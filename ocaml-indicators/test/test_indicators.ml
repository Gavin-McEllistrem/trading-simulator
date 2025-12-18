(** Test suite for technical indicators *)

open Indicators

let float_equal ~epsilon a b =
  abs_float (a -. b) < epsilon

let array_equal ~epsilon arr1 arr2 =
  Array.length arr1 = Array.length arr2 &&
  Array.for_all2 (float_equal ~epsilon) arr1 arr2

let test_average () =
  Printf.printf "Testing average...\n";
  assert (float_equal ~epsilon:0.001 (average [|1.0; 2.0; 3.0; 4.0; 5.0|]) 3.0);
  assert (float_equal ~epsilon:0.001 (average [|10.0; 20.0; 30.0|]) 20.0);
  assert (float_equal ~epsilon:0.001 (average [||]) 0.0);
  Printf.printf "  ✓ average tests passed\n"

let test_std_dev () =
  Printf.printf "Testing std_dev...\n";
  (* Sample: [2, 4, 4, 4, 5, 5, 7, 9] -> mean=5, variance=4, std_dev=2 *)
  let data = [|2.0; 4.0; 4.0; 4.0; 5.0; 5.0; 7.0; 9.0|] in
  assert (float_equal ~epsilon:0.001 (std_dev data) 2.0);
  assert (float_equal ~epsilon:0.001 (std_dev [||]) 0.0);
  assert (float_equal ~epsilon:0.001 (std_dev [|5.0|]) 0.0);
  Printf.printf "  ✓ std_dev tests passed\n"

let test_sliding_window () =
  Printf.printf "Testing sliding_window...\n";
  let data = [|1.0; 2.0; 3.0; 4.0; 5.0|] in
  let windows = sliding_window data 3 in
  assert (Array.length windows = 3);
  assert (array_equal ~epsilon:0.001 windows.(0) [|1.0; 2.0; 3.0|]);
  assert (array_equal ~epsilon:0.001 windows.(1) [|2.0; 3.0; 4.0|]);
  assert (array_equal ~epsilon:0.001 windows.(2) [|3.0; 4.0; 5.0|]);
  Printf.printf "  ✓ sliding_window tests passed\n"

let test_sma () =
  Printf.printf "Testing SMA...\n";
  let data = [|1.0; 2.0; 3.0; 4.0; 5.0|] in
  let result = sma data 3 in
  assert (Array.length result = 3);
  assert (float_equal ~epsilon:0.001 result.(0) 2.0);  (* avg(1,2,3) *)
  assert (float_equal ~epsilon:0.001 result.(1) 3.0);  (* avg(2,3,4) *)
  assert (float_equal ~epsilon:0.001 result.(2) 4.0);  (* avg(3,4,5) *)
  Printf.printf "  ✓ SMA tests passed\n"

let test_ema () =
  Printf.printf "Testing EMA...\n";
  let data = [|1.0; 2.0; 3.0; 4.0; 5.0; 6.0; 7.0; 8.0; 9.0; 10.0|] in
  let result = ema data 3 in
  assert (Array.length result = Array.length data);
  (* First period-1 elements should be seed (SMA of first 3) *)
  let seed = 2.0 in  (* avg(1,2,3) *)
  assert (float_equal ~epsilon:0.001 result.(0) seed);
  assert (float_equal ~epsilon:0.001 result.(1) seed);
  (* EMA should be increasing for this monotonic data *)
  assert (result.(9) > result.(2));
  Printf.printf "  ✓ EMA tests passed\n"

let test_rsi () =
  Printf.printf "Testing RSI...\n";
  (* Create data with clear uptrend then downtrend *)
  let data = [|
    44.0; 44.5; 45.0; 45.5; 46.0; 46.5; 47.0;  (* uptrend *)
    46.5; 46.0; 45.5; 45.0; 44.5; 44.0; 43.5;  (* downtrend *)
  |] in
  let result = rsi data 6 in
  assert (Array.length result = Array.length data);
  (* During warmup period, RSI should be neutral (50.0) *)
  for i = 0 to 5 do
    assert (float_equal ~epsilon:0.001 result.(i) 50.0);
  done;
  (* After uptrend, RSI should be > 50 *)
  assert (result.(6) > 50.0);
  (* After downtrend, RSI should be < 50 *)
  assert (result.(13) < 50.0);
  Printf.printf "  ✓ RSI tests passed\n"

let test_macd () =
  Printf.printf "Testing MACD...\n";
  let data = Array.init 50 (fun i -> 100.0 +. float_of_int i) in
  let (macd_line, signal_line, histogram) = macd data 12 26 9 in
  assert (Array.length macd_line = Array.length data);
  assert (Array.length signal_line = Array.length data);
  assert (Array.length histogram = Array.length data);
  (* For uptrending data, MACD line should generally be positive *)
  assert (macd_line.(49) > 0.0);
  Printf.printf "  ✓ MACD tests passed\n"

let test_bollinger_bands () =
  Printf.printf "Testing Bollinger Bands...\n";
  let data = [|
    100.0; 101.0; 102.0; 103.0; 104.0;
    105.0; 106.0; 107.0; 108.0; 109.0;
  |] in
  let (upper, middle, lower) = bollinger_bands data 5 2.0 in
  assert (Array.length upper = Array.length data);
  assert (Array.length middle = Array.length data);
  assert (Array.length lower = Array.length data);
  (* Middle band should be SMA *)
  (* Upper band should be above middle *)
  (* Lower band should be below middle *)
  for i = 4 to 9 do
    assert (upper.(i) > middle.(i));
    assert (lower.(i) < middle.(i));
  done;
  Printf.printf "  ✓ Bollinger Bands tests passed\n"

let () =
  Printf.printf "\n=== Running Indicator Tests ===\n\n";
  test_average ();
  test_std_dev ();
  test_sliding_window ();
  test_sma ();
  test_ema ();
  test_rsi ();
  test_macd ();
  test_bollinger_bands ();
  Printf.printf "\n✅ All tests passed!\n\n"
