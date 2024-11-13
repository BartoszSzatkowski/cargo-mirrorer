use super::constants::{DEFAULT_CONFIG, DEFAULT_INDEX_PATH};
use super::plan::FetchPlan;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use tracing::info;

pub struct IndexConfig {
    pub fetch_plan: FetchPlan,
    pub index_path: Option<PathBuf>,
}

pub struct Index {
    pub ready: bool,
    pub config: IndexConfig,
}

impl Default for Index {
    fn default() -> Self {
        Self::new(IndexConfig {
            fetch_plan: FetchPlan::AllCrates,
            index_path: Some(PathBuf::from(DEFAULT_INDEX_PATH)),
        })
    }
}

impl Index {
    pub fn new(c: IndexConfig) -> Self {
        Self {
            ready: false,
            config: c,
        }
    }

    // pub fn is_index_present() -> bool {}
    // pub fn adheres_to_fetching_policy() -> bool {}
    // pub fn check_readiness() {}

    pub fn clone_full_main_index() {
        Command::new("git")
            .arg("clone")
            .arg("--depth=1")
            .arg("https://github.com/rust-lang/crates.io-index.git")
            .arg(DEFAULT_INDEX_PATH)
            .output()
            .expect("failed to fetch crates.io-index repo (git not available?)");
    }
}

pub async fn download_index_of_crates() -> anyhow::Result<()> {
    info!("Start downloading crates io index");
    let start = Instant::now();
    Command::new("git")
        .arg("clone")
        .arg("--depth=1")
        .arg("https://github.com/rust-lang/crates.io-index.git")
        .arg(DEFAULT_INDEX_PATH)
        .output()
        .expect("failed to fetch crates.io-index repo (git not available?)");
    // create a file that allows the report to be used by git http server
    File::create(format!("./{DEFAULT_INDEX_PATH}/.git/git-daemon-export-ok"))?;
    // replace the config file in the base of the index
    let mut conf_file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(format!("./{DEFAULT_INDEX_PATH}/config.json"))?;
    conf_file.write_all(DEFAULT_CONFIG.as_bytes())?;
    // create first commit
    Command::new("git")
        .current_dir(format!("./{DEFAULT_INDEX_PATH}"))
        .arg("add")
        .arg(".")
        .output()
        .expect("failed to stage git changes");
    Command::new("git")
        .current_dir(format!("./{DEFAULT_INDEX_PATH}"))
        .arg("commit")
        .arg("-m")
        .arg(r#""chore: modify config.json""#)
        .output()
        .expect("failed to create a git commit");
    info!(
        "Crates io index on the file system in: {:?}",
        start.elapsed()
    );

    Ok(())
}
