#[derive(Debug, PartialEq)]
pub enum FetchPlan {
    AllCrates,
    Playground,
    TopN(usize),
    List(Vec<String>),
}

impl TryFrom<String> for FetchPlan {
    type Error = String;

    fn try_from(value: String) -> anyhow::Result<Self, Self::Error> {
        match value.as_str() {
            "all" => Ok(Self::AllCrates),
            "playground" => Ok(Self::Playground),
            n if n.parse::<usize>().is_ok() => Ok(Self::TopN(n.parse::<usize>().unwrap())),
            l if l.contains(',') => Ok(Self::List(l.split(",").map(String::from).collect())),
            _ => Err("Invalid fetching strategy provided".to_string()),
        }
    }
}

pub async fn download_index_of_crates() -> anyhow::Result<()> {
    info!("Start downloading crates io index");
    let start = Instant::now();
    let body = reqwest::get("https://github.com/rust-lang/crates.io-index/tarball/master")
        .await?
        .bytes()
        .await?
        .to_vec();
    let tar = GzDecoder::new(&body[..]);
    let mut archive = Archive::new(tar);
    archive.unpack("crates-index")?;
    info!(
        "Crates io index on the file system in: {:?}",
        start.elapsed()
    );

    Ok(())
}

#[derive(Debug)]
pub struct Krate {
    path: String,
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
        for entry in WalkDir::new("./crates-index")
            .min_depth(3)
            .max_depth(4)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
            .filter(is_file)
        {
            scope.spawn(|_scope| {
                let entry = entry.unwrap();
                let path = entry.path();
            if let Ok(lines) = read_lines(path) {
                // Consumes the iterator, returns an (Optional) String
                for line in lines.map_while(Result::ok) {
                    let nodes = unsafe { sonic_rs::get_many_unchecked(&line, &tree) };
                    let Ok(nodes) = nodes else {
                        continue;
                    };
                    let crate_name = nodes[0].as_str().unwrap();
                    let crate_version = nodes[1].as_str().unwrap();
                    let krate = Krate {
                        path: format!("./crates/{}-{}.crate", crate_name, crate_version),
                        url: format!(
                                "https://static.crates.io/crates/{crate_name}/{crate_name}-{crate_version}.crate",
                            )
                    };
                    krates.lock().push(krate);
                }
            }});
        }
    });
    info!("Crates info extracted in: {:?}", start.elapsed());

    let v = krates.into_inner().into_iter().map(download_crate);
    futures::stream::iter(v)
        .buffer_unordered(5)
        .collect::<Vec<_>>()
        .await;

    Ok(())
}

async fn download_crate(krate: Krate) -> anyhow::Result<()> {
    let body = reqwest::get(krate.url.clone()).await?.bytes().await?;
    std::fs::write(krate.path.clone(), body)?;

    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn is_file(dir_entry: &Result<walkdir::DirEntry, walkdir::Error>) -> bool {
    let Ok(dir_entry) = dir_entry else {
        return false;
    };
    let Ok(ent) = dir_entry.metadata() else {
        return false;
    };
    ent.is_file()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_parses_fetching_flag_correctly() {
        assert_eq!(
            Ok(FetchPlan::AllCrates),
            FetchPlan::try_from("all".to_string())
        );
        assert_eq!(
            Ok(FetchPlan::Playground),
            FetchPlan::try_from("playground".to_string())
        );
        assert_eq!(
            Ok(FetchPlan::TopN(100)),
            FetchPlan::try_from("100".to_string())
        );
        assert_eq!(
            Ok(FetchPlan::List(vec![
                "tokio".to_string(),
                "serde".to_string()
            ])),
            FetchPlan::try_from("tokio,serde".to_string())
        );
        assert_eq!(
            Err("Invalid fetching strategy provided".to_string()),
            FetchPlan::try_from("v9W1x".to_string())
        );
    }
}
