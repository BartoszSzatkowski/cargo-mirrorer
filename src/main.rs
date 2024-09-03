use cargo_mirrorer::fetching::FetchPlan;
use cargo_mirrorer::serving::ServingPlan;
use clap::Parser;
use flate2::bufread::GzDecoder;
use futures::stream::StreamExt;
use parking_lot::Mutex;
use rayon::ThreadPoolBuilder;
use sonic_rs::JsonValueTrait;
use std::fs::File;
use std::io::{self, BufRead};
use std::num::NonZeroUsize;
use std::path::Path;
use std::thread;
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
    /// Protocol to use for serving info about crates
    #[arg(short = 'S', long = "serve", value_name = "git|sparse")]
    pub fetch_plan: String,
    /// Crates that should be fetched for mirrorer to host
    #[arg(
        short = 'F',
        long = "fetch",
        value_name = "all|playground|n<number>|<comma separated list>"
    )]
    pub fetch_plan: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger();

    let config = Config::parse();

    match FetchPlan::try_from(config.fetch_plan)? {
        FetchPlan::AllCrates => {
            // download_index_of_crates().await?;
            // download_crates_in_index().await?;
        }
        _ => todo!(),
    }

    match ServingPlan::try_from(config.serving_plan)? {
        ServingPlan::Git => {}
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
