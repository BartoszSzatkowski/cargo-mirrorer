use super::constants::{DEFAULT_CONFIG, DEFAULT_INDEX_PATH, DEFAULT_SOURCE_INDEX};
use super::plan::FetchPlan;
use anyhow::{anyhow, Context};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use tracing::info;

pub struct IndexConfig {
    pub fetch_plan: FetchPlan,
    pub index_path: PathBuf,
    pub source_index: String,
}

pub enum IndexState {
    Ready,
    OutOfDate,
    Invalid,
}

pub struct Index {
    pub ready: bool,
    pub config: IndexConfig,
}

impl Default for Index {
    fn default() -> Self {
        Self::new(IndexConfig {
            fetch_plan: FetchPlan::AllCrates,
            index_path: PathBuf::from(DEFAULT_INDEX_PATH),
            source_index: DEFAULT_SOURCE_INDEX.to_string(),
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

    pub fn adheres_to_fetching_policy(&self) -> anyhow::Result<bool> {
        if !self.config.index_path.exists() {
            return Ok(false);
        };

        match &self.config.fetch_plan {
            FetchPlan::AllCrates => self.check_index_main()?,
            FetchPlan::List(l) => {
                self.check_index_lean(l)?;
            }
            _ => todo!(),
        }

        Ok(true)
    }

    fn check_index_main(&self) -> anyhow::Result<()> {
        let out = Command::new("git")
            .current_dir(&self.config.index_path)
            .arg("config")
            .arg("--get")
            .arg("remote.origin.url")
            .output()
            .context("failed to extract origin url from index repo")?;

        // TODO: check when was the last pull performed by inspecting .git/FEATCH_HEAD file

        if String::from_utf8_lossy(&out.stdout) != self.config.source_index {
            return Err(anyhow!(
                "Origin of crate index is not matching specified source index [default: {}]",
                DEFAULT_SOURCE_INDEX
            ));
        }

        Ok(())
    }

    fn check_index_lean(&self, required_cratres: &Vec<String>) -> anyhow::Result<()> {
        todo!()
    }

    fn exists_in_index(&self, krate_name: &str) -> bool {
        let index_display = self.config.index_path.display();
        let path_to_krate = match krate_name.len() {
            1 => PathBuf::from(format!("{}/1/{}", index_display, krate_name)),
            2 => PathBuf::from(format!("{}/2/{}", index_display, krate_name)),
            3 => {
                let first_letter: String =
                    krate_name.to_string().chars().nth(0).into_iter().collect();
                PathBuf::from(format!(
                    "{}/3/{}/{}",
                    index_display, first_letter, krate_name
                ))
            }
            _ => {
                let c = krate_name.to_string();
                let first_two: String = c.chars().take(2).collect();
                let second_two: String = c.chars().skip(2).take(2).collect();

                PathBuf::from(format!(
                    "{}/{}/{}/{}",
                    index_display, first_two, second_two, krate_name
                ))
            }
        };

        path_to_krate.is_file()
    }

    pub fn clone_full_main_index(&self) -> anyhow::Result<()> {
        Command::new("git")
            .arg("clone")
            .arg("--depth=1")
            .arg(&self.config.source_index)
            .arg(&self.config.index_path)
            .output()
            .context("failed to clone crates.io-index repo")?;

        Ok(())
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
