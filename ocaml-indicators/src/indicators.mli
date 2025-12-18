(** Technical Indicator Library

    Pure functional implementations of common technical indicators
    for trading systems. All functions are side-effect free and
    designed for easy FFI integration with Rust.
*)

(** {1 Core Types} *)

(** Represents a time series of prices *)
type price_data = float array

(** {1 Helper Functions} *)

(** [average arr] computes the arithmetic mean of all values in [arr].
    Returns 0.0 for empty arrays. *)
val average : float array -> float

(** [std_dev arr] computes the standard deviation of values in [arr].
    Returns 0.0 for arrays with fewer than 2 elements. *)
val std_dev : float array -> float

(** [sliding_window data window_size] creates overlapping windows of size [window_size]
    from [data]. Returns an array of windows (each window is an array). *)
val sliding_window : 'a array -> int -> 'a array array

(** {1 Moving Averages} *)

(** [sma data period] computes the Simple Moving Average with the given [period].
    Returns an array of length [length data - period + 1].
    Each element is the average of [period] consecutive values.

    Example: [sma [|1.0; 2.0; 3.0; 4.0; 5.0|] 3] returns [[|2.0; 3.0; 4.0|]]

    @raise Invalid_argument if [period <= 0] or [period > length data] *)
val sma : price_data -> int -> float array

(** [ema data period] computes the Exponential Moving Average with the given [period].
    Uses smoothing factor: alpha = 2 / (period + 1).
    Returns an array of the same length as [data], with first [period-1] elements
    set to the initial SMA seed value.

    @raise Invalid_argument if [period <= 0] or [period > length data] *)
val ema : price_data -> int -> float array

(** {1 Momentum Indicators} *)

(** [rsi data period] computes the Relative Strength Index.
    Returns an array of RSI values in range 0.0-100.0.
    The first [period] values will be 50.0 (neutral) as warmup.

    RSI = 100 - (100 / (1 + RS))
    where RS = Average Gain / Average Loss over the period

    @raise Invalid_argument if [period <= 0] or [period > length data] *)
val rsi : price_data -> int -> float array

(** [macd data fast_period slow_period signal_period] computes MACD indicator.
    Returns a tuple [(macd_line, signal_line, histogram)] where:
    - macd_line = EMA(fast) - EMA(slow)
    - signal_line = EMA(macd_line, signal_period)
    - histogram = macd_line - signal_line

    All arrays have the same length as [data].

    @raise Invalid_argument if any period is invalid or periods are not ordered properly *)
val macd : price_data -> int -> int -> int -> (float array * float array * float array)

(** {1 Volatility Indicators} *)

(** [bollinger_bands data period num_std_dev] computes Bollinger Bands.
    Returns a tuple [(upper_band, middle_band, lower_band)] where:
    - middle_band = SMA(data, period)
    - upper_band = middle_band + (std_dev * num_std_dev)
    - lower_band = middle_band - (std_dev * num_std_dev)

    The first [period-1] elements will have bands equal to the price.

    @raise Invalid_argument if [period <= 0] or [num_std_dev <= 0.0] *)
val bollinger_bands : price_data -> int -> float -> (float array * float array * float array)
