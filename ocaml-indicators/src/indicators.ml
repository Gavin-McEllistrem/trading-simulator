(** Technical Indicator Library - Implementation *)

type price_data = float array

(** {1 Helper Functions} *)

let average arr =
  if Array.length arr = 0 then 0.0
  else
    let sum = Array.fold_left (+.) 0.0 arr in
    sum /. float_of_int (Array.length arr)

let std_dev arr =
  let len = Array.length arr in
  if len < 2 then 0.0
  else
    let mean = average arr in
    let variance_sum = Array.fold_left (fun acc x ->
      let diff = x -. mean in
      acc +. (diff *. diff)
    ) 0.0 arr in
    sqrt (variance_sum /. float_of_int len)

let sliding_window data window_size =
  let len = Array.length data in
  if window_size <= 0 || window_size > len then
    [||]
  else
    let num_windows = len - window_size + 1 in
    Array.init num_windows (fun i ->
      Array.sub data i window_size
    )

(** {1 Moving Averages} *)

let sma data period =
  if period <= 0 then
    invalid_arg "sma: period must be positive"
  else if period > Array.length data then
    invalid_arg "sma: period cannot exceed data length"
  else
    let windows = sliding_window data period in
    Array.map average windows

let ema data period =
  let len = Array.length data in
  if period <= 0 then
    invalid_arg "ema: period must be positive"
  else if period > len then
    invalid_arg "ema: period cannot exceed data length"
  else
    let alpha = 2.0 /. (float_of_int period +. 1.0) in
    let result = Array.make len 0.0 in

    (* Initialize with SMA of first 'period' elements *)
    let initial_window = Array.sub data 0 period in
    let seed = average initial_window in

    (* Fill warmup period with seed value *)
    for i = 0 to period - 2 do
      result.(i) <- seed
    done;
    result.(period - 1) <- seed;

    (* Compute EMA for remaining elements *)
    for i = period to len - 1 do
      result.(i) <- alpha *. data.(i) +. (1.0 -. alpha) *. result.(i - 1)
    done;

    result

(** {1 Momentum Indicators} *)

let rsi data period =
  let len = Array.length data in
  if period <= 0 then
    invalid_arg "rsi: period must be positive"
  else if period >= len then
    invalid_arg "rsi: period must be less than data length"
  else
    let result = Array.make len 50.0 in  (* Neutral RSI during warmup *)

    (* Calculate price changes *)
    let changes = Array.init (len - 1) (fun i -> data.(i + 1) -. data.(i)) in

    (* Separate gains and losses *)
    let gains = Array.map (fun x -> if x > 0.0 then x else 0.0) changes in
    let losses = Array.map (fun x -> if x < 0.0 then abs_float x else 0.0) changes in

    (* Calculate initial average gain and loss *)
    let initial_avg_gain =
      let sum = ref 0.0 in
      for i = 0 to period - 1 do
        sum := !sum +. gains.(i)
      done;
      !sum /. float_of_int period
    in

    let initial_avg_loss =
      let sum = ref 0.0 in
      for i = 0 to period - 1 do
        sum := !sum +. losses.(i)
      done;
      !sum /. float_of_int period
    in

    (* Calculate RSI using smoothed averages *)
    let avg_gain = ref initial_avg_gain in
    let avg_loss = ref initial_avg_loss in

    for i = period to len - 1 do
      let rs = if !avg_loss = 0.0 then 100.0 else !avg_gain /. !avg_loss in
      result.(i) <- 100.0 -. (100.0 /. (1.0 +. rs));

      (* Update smoothed averages for next iteration *)
      if i < len - 1 then (
        avg_gain := (!avg_gain *. float_of_int (period - 1) +. gains.(i)) /. float_of_int period;
        avg_loss := (!avg_loss *. float_of_int (period - 1) +. losses.(i)) /. float_of_int period;
      )
    done;

    result

let macd data fast_period slow_period signal_period =
  if fast_period <= 0 || slow_period <= 0 || signal_period <= 0 then
    invalid_arg "macd: all periods must be positive"
  else if fast_period >= slow_period then
    invalid_arg "macd: fast_period must be less than slow_period"
  else if slow_period > Array.length data then
    invalid_arg "macd: slow_period cannot exceed data length"
  else
    let fast_ema = ema data fast_period in
    let slow_ema = ema data slow_period in

    (* MACD Line = Fast EMA - Slow EMA *)
    let macd_line = Array.init (Array.length data) (fun i ->
      fast_ema.(i) -. slow_ema.(i)
    ) in

    (* Signal Line = EMA of MACD Line *)
    let signal_line = ema macd_line signal_period in

    (* Histogram = MACD Line - Signal Line *)
    let histogram = Array.init (Array.length data) (fun i ->
      macd_line.(i) -. signal_line.(i)
    ) in

    (macd_line, signal_line, histogram)

(** {1 Volatility Indicators} *)

let bollinger_bands data period num_std_dev =
  let len = Array.length data in
  if period <= 0 then
    invalid_arg "bollinger_bands: period must be positive"
  else if num_std_dev <= 0.0 then
    invalid_arg "bollinger_bands: num_std_dev must be positive"
  else if period > len then
    invalid_arg "bollinger_bands: period cannot exceed data length"
  else
    let middle_band = Array.make len 0.0 in
    let upper_band = Array.make len 0.0 in
    let lower_band = Array.make len 0.0 in

    (* Fill warmup period with actual prices *)
    for i = 0 to period - 2 do
      middle_band.(i) <- data.(i);
      upper_band.(i) <- data.(i);
      lower_band.(i) <- data.(i);
    done;

    (* Calculate Bollinger Bands for valid windows *)
    let windows = sliding_window data period in
    Array.iteri (fun i window ->
      let idx = i + period - 1 in
      let mean = average window in
      let std = std_dev window in
      middle_band.(idx) <- mean;
      upper_band.(idx) <- mean +. (num_std_dev *. std);
      lower_band.(idx) <- mean -. (num_std_dev *. std);
    ) windows;

    (upper_band, middle_band, lower_band)
