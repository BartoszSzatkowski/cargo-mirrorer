use cargo_mirrorer::fetching::FetchPlan;
use clap::Parser;
use flate2::bufread::GzDecoder;
use std::time::Instant;
use tar::Archive;
use tracing::info;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use walkdir::{DirEntry, WalkDir};

// Ideas for reducing amount of downloaded data:
//
// http query for most downloaded crates: https://crates.io/api/v1/crates?per_page=100&page=1&sort=recent-downloads
// curated list crates from rust playground: https://github.com/rust-lang/rust-playground/blob/9ba74ff/compiler/base/Cargo.toml
// download tarball of crates io index: https://github.com/rust-lang/crates.io-index/tarball/master
//
// Currently whole index takes few minutes to download

/// Mirror crates io
#[derive(Debug, Parser)]
pub struct Config {
    /// Directory where downloaded .crate files will be saved to.
    #[arg(short = 'F', long = "fetch-plan", value_name = "PLAN")]
    pub fetch_plan: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger();

    let config = Config::parse();

    match FetchPlan::try_from(config.fetch_plan).unwrap() {
        FetchPlan::AllCrates => {
            // download_crates_io_index().await?;
            for entry in WalkDir::new("./crates-index").min_depth(3).max_depth(3) {
                println!("{}", entry?.path().display());
            }
        }
        _ => todo!(),
    }

    Ok(())
}

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

async fn download_crates_io_index() -> anyhow::Result<()> {
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
