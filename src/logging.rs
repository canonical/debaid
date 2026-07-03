use tracing_subscriber::{EnvFilter, fmt};

/// Diagnostics go to stderr so stdout stays free for the `detect`/`verify` JSON
/// snapshots. `RUST_LOG` overrides the `-v`-derived level
pub fn init(verbose: u8) {
    let default = match verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));
    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}
