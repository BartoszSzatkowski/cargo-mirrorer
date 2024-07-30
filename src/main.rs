use tracing_subscriber::filter::{EnvFilter, LevelFilter};

fn init_logger() {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time()
        .with_target(false)
        .init();
}

fn main() -> anyhow::Result<()> {
    init_logger();

    Ok(())
}
