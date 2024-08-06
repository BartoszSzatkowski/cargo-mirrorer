use flate2::bufread::GzDecoder;
use std::time::Instant;
use tar::Archive;
use tracing::info;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

// Ideas for reducing amount of downloaded data:
//
// http query for most downloaded crates: https://crates.io/api/v1/crates?per_page=100&page=1&sort=recent-downloads
// curated list crates from rust playground: https://github.com/rust-lang/rust-playground/blob/9ba74ff/compiler/base/Cargo.toml
// download tarball of crates io index: https://github.com/rust-lang/crates.io-index/tarball/master
//
// Currently whole index takes few minutes to donwload

fn init_logger() {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .without_time()
        .with_target(false)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger();

    info!("Start downloading crates io index");
    let start = Instant::now();
    let body = reqwest::get("https://github.com/rust-lang/crates.io-index/tarball/master")
        .await?
        .bytes()
        .await?
        .to_vec();
    let tar = GzDecoder::new(&body[..]);
    let mut archive = Archive::new(tar);
    archive.unpack(".")?;
    info!(
        "Crates io index on the file system in: {:?}",
        start.elapsed()
    );

    Ok(())
}
