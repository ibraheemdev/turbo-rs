use std::time::Duration;

pub struct Config {
  pub(crate) http1_keepalive: bool,
  pub(crate) http1_half_close: bool,
  pub(crate) http1_max_buf_size: Option<usize>,
  pub(crate) http1_writev: Option<bool>,
  pub(crate) http1_only: bool,
  pub(crate) http2_only: bool,
  pub(crate) http2_initial_stream_window_size: Option<u32>,
  pub(crate) http2_initial_connection_window_size: Option<u32>,
  pub(crate) http2_adaptive_window: bool,
  pub(crate) http2_max_frame_size: Option<u32>,
  pub(crate) http2_max_concurrent_streams: Option<u32>,
  pub(crate) http2_keep_alive_interval: Option<Duration>,
  pub(crate) http2_keep_alive_timeout: Duration,
  pub(crate) tcp_keepalive: Option<Duration>,
  pub(crate) tcp_nodelay: bool,
  pub(crate) tcp_sleep_on_accept_errors: bool,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      http1_keepalive: true,
      http1_half_close: false,
      http1_max_buf_size: None,
      http1_writev: None,
      http1_only: false,
      http2_only: false,
      http2_initial_stream_window_size: None,
      http2_initial_connection_window_size: None,
      http2_adaptive_window: false,
      http2_max_frame_size: None,
      http2_max_concurrent_streams: None,
      http2_keep_alive_interval: None,
      http2_keep_alive_timeout: Duration::from_secs(20),
      tcp_keepalive: None,
      tcp_nodelay: false,
      tcp_sleep_on_accept_errors: true,
    }
  }
}

impl Config {
  /// Sets whether to use keep-alive for HTTP/1 connections.
  ///
  /// Default is true.
  pub fn http1_keepalive(mut self, val: bool) -> Self {
    self.http1_keepalive = val;
    self
  }
  /// Set whether HTTP/1 connections should support half-closures.
  ///
  /// Clients can chose to shutdown their write-side while waiting for the server to respond. Setting
  /// this to true will prevent closing the connection immediately if read detects an EOF in the
  /// middle of a request.
  ///
  /// Default is false.
  pub fn http1_half_close(mut self, val: bool) -> Self {
    self.http1_half_close = val;
    self
  }
  /// Set the maximum buffer size.
  ///
  /// Default is ~ 400kb.
  pub fn http1_max_buf_size(mut self, val: usize) -> Self {
    self.http1_max_buf_size = Some(val);
    self
  }
  /// Set whether HTTP/1 connections should try to use vectored writes, or always flatten into a
  /// single buffer.
  pub fn http1_writev(mut self, val: bool) -> Self {
    self.http1_writev = Some(val);
    self
  }

  /// Sets whether HTTP/1 is required.
  ///
  /// Default is false.
  pub fn http1_only(mut self, val: bool) -> Self {
    self.http1_only = val;
    self
  }

  /// Sets whether HTTP/2 is required.
  ///
  /// Default is false.
  pub fn http2_only(mut self, val: bool) -> Self {
    self.http2_only = val;
    self
  }

  /// Sets the `SETTINGS_INITIAL_WINDOW_SIZE` option for HTTP2 stream-level flow control.
  ///
  /// Passing None will do nothing.
  ///
  /// If not set, hyper will use a default.
  pub fn http2_initial_stream_window_size(mut self, sz: impl Into<Option<u32>>) -> Self {
    self.http2_initial_stream_window_size = sz.into();
    self
  }

  /// Sets the max connection-level flow control for HTTP2
  ///
  /// Passing `None` will do nothing.
  ///
  /// If not set, hyper will use a default.
  pub fn http2_initial_connection_window_size(mut self, sz: impl Into<Option<u32>>) -> Self {
    self.http2_initial_connection_window_size = sz.into();
    self
  }

  /// Sets whether to use an adaptive flow control.
  ///
  /// Enabling this will override the limits set in `http2_initial_stream_window_size` and
  /// `http2_initial_connection_window_size`.
  pub fn http2_adaptive_window(mut self, enabled: bool) -> Self {
    self.http2_adaptive_window = enabled;
    self
  }

  /// Sets the maximum frame size to use for HTTP2.
  ///
  /// Passing `None` will do nothing.
  ///
  /// If not set, hyper will use a default.
  pub fn http2_max_frame_size(mut self, sz: impl Into<Option<u32>>) -> Self {
    self.http2_max_frame_size = sz.into();
    self
  }

  /// Sets the `SETTINGS_MAX_CONCURRENT_STREAMS` option for HTTP2 connections.
  ///
  /// Default is no limit (std::u32::MAX). Passing `None` will do nothing.
  pub fn http2_max_concurrent_streams(mut self, max: impl Into<Option<u32>>) -> Self {
    self.http2_max_concurrent_streams = max.into();
    self
  }

  /// Sets an interval for HTTP2 Ping frames should be sent to keep a connection alive.
  ///
  /// Pass `None` to disable HTTP2 keep-alive.
  ///
  /// Default is currently disabled.
  pub fn http2_keep_alive_interval(mut self, interval: impl Into<Option<Duration>>) -> Self {
    self.http2_keep_alive_interval = interval.into();
    self
  }

  /// Sets a timeout for receiving an acknowledgement of the keep-alive ping.
  ///
  /// If the ping is not acknowledged within the timeout, the connection will be closed. Does
  /// nothing if `http2_keep_alive_interval` is disabled.
  ///
  /// Default is 20 seconds.
  pub fn http2_keep_alive_timeout(mut self, timeout: Duration) -> Self {
    self.http2_keep_alive_timeout = timeout;
    self
  }

  /// Set whether TCP keepalive messages are enabled on accepted connections.
  ///
  /// If `None` is specified, keepalive is disabled, otherwise the duration specified will be the
  /// time to remain idle before sending TCP keepalive probes.
  ///
  pub fn tcp_keepalive(mut self, keepalive: Option<Duration>) -> Self {
    self.tcp_keepalive = keepalive;
    self
  }

  /// Set the value of `TCP_NODELAY` option for accepted connections.
  pub fn tcp_nodelay(mut self, enabled: bool) -> Self {
    self.tcp_nodelay = enabled;
    self
  }

  /// Set whether to sleep on accept errors.
  ///
  /// A possible scenario is that the process has hit the max open files
  /// allowed, and so trying to accept a new connection will fail with
  /// EMFILE. In some cases, it's preferable to just wait for some time, if
  /// the application will likely close some files (or connections), and try
  /// to accept the connection again. If this option is true, the error will
  /// be logged at the error level, since it is still a big deal, and then
  /// the listener will sleep for 1 second.
  ///
  /// In other cases, hitting the max open files should be treat similarly
  /// to being out-of-memory, and simply error (and shutdown). Setting this
  /// option to false will allow that.
  ///
  /// Default is true.
  pub fn tcp_sleep_on_accept_errors(mut self, val: bool) -> Self {
    self.tcp_sleep_on_accept_errors = val;
    self
  }
}
