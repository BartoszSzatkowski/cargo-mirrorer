use cargo_mirrorer::fetching::FetchPlan;
use clap::Parser;
use flate2::bufread::GzDecoder;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::Instant;
use tar::Archive;
use tracing::info;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use walkdir::WalkDir;

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

    let mut tree = sonic_rs::PointerTree::new();
    tree.add_path(&["name"]);
    tree.add_path(&["vers"]);

    match FetchPlan::try_from(config.fetch_plan).unwrap() {
        FetchPlan::AllCrates => {
            // download_crates_io_index().await?;
            for entry in WalkDir::new("./crates-index")
                .min_depth(3)
                .max_depth(3)
                .into_iter()
                .take(5)
            {
                if let Ok(lines) = read_lines(entry?.path()) {
                    // Consumes the iterator, returns an (Optional) String
                    for line in lines.map_while(Result::ok) {
                        let nodes = unsafe { sonic_rs::get_many_unchecked(&line, &tree) };
                        let nodes = nodes.unwrap();
                        println!("nodes 1 {}", nodes[0]);
                        println!("nodes 2 {}", nodes[1]);
                    }
                }
                // let body =
                //     reqwest::get("https://github.com/rust-lang/crates.io-index/tarball/master")
                //         .await?
                //         .bytes()
                //         .await?
                //         .to_vec();
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

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
