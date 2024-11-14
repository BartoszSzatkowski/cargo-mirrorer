use super::constants::{DEFAULT_CRATES_PATH, DEFAULT_INDEX_PATH};
use super::util::{is_file, is_hidden};
use futures::stream::StreamExt;
use parking_lot::Mutex;
use rayon::ThreadPoolBuilder;
use sonic_rs::JsonValueTrait;
use std::fs::{self, File};
use std::io::{self, BufRead};
use std::num::NonZeroUsize;
use std::path::Path;
use std::thread;
use std::time::Instant;
use tracing::info;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct KratePath {
    name: String,
    version: String,
}

impl std::fmt::Display for KratePath {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "./crates/{}-{}.crate", self.name, self.version)
    }
}

#[derive(Debug)]
pub struct Krate {
    path: KratePath,
    url: String,
}

pub async fn download_crates_in_index() -> anyhow::Result<()> {
    let num_threads = thread::available_parallelism().map_or(1, NonZeroUsize::get);
    let thread_pool = ThreadPoolBuilder::new().num_threads(num_threads).build()?;

    let krates = Mutex::new(Vec::with_capacity(1_500_000));

    let mut tree = sonic_rs::PointerTree::new();
    tree.add_path(&["name"]);
    tree.add_path(&["vers"]);

    let start = Instant::now();

    info!("Start extracting data about crates");
    thread_pool.in_place_scope(|scope| {
        for entry in WalkDir::new(format!("./{DEFAULT_INDEX_PATH}"))
            .min_depth(3)
            .max_depth(4)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
            .filter(is_file)
        {
            info!("Found a file!");
            scope.spawn(|_scope| {
                let entry = entry.unwrap();
                dbg!(entry.clone());
                let path = entry.path();
                if let Ok(lines) = read_lines(path) {
                    // Consumes the iterator, returns an (Optional) String
                    for line in lines.map_while(Result::ok) {
                        let nodes = unsafe { sonic_rs::get_many_unchecked(&line, &tree) };
                        let Ok(nodes) = nodes else {
                            continue;
                        };
                        let name = nodes[0].as_str().unwrap().to_string();
                        let version = nodes[1].as_str().unwrap().to_string();
                        let krate = Krate {
                            path: KratePath {
                                name: name.clone(),
                                version: version.clone(),
                            },
                            url: format!(
                                "https://static.crates.io/crates/{name}/{name}-{version}.crate",
                            ),
                        };
                        krates.lock().push(krate);
                    }
                }
            });
        }
    });
    info!("Crates info extracted in: {:?}", start.elapsed());

    fs::create_dir_all(DEFAULT_CRATES_PATH)?;
    let v = krates.into_inner().into_iter().map(download_crate);
    futures::stream::iter(v)
        .buffer_unordered(5)
        .collect::<Vec<_>>()
        .await;
    info!("Crates downloaded in: {:?}", start.elapsed());

    Ok(())
}

async fn download_crate(krate: Krate) -> anyhow::Result<()> {
    let body = reqwest::get(krate.url.clone()).await?.bytes().await?;
    std::fs::write(krate.path.to_string(), body)?;
    info!("a crate was downloaded!");

    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
