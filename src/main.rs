use gix::diff::object::bstr::BStr;
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

    let url = gix::url::parse(BStr::new(
        "https://github.com/rust-lang/crates.io-index.git",
    ))?;

    println!("Url: {:?}", url.to_bstring());
    let mut prepare_clone = gix::prepare_clone(url, &"./banana")?;

    println!("Cloning crates index into here ...");
    let (mut prepare_checkout, _) = prepare_clone
        .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

    println!(
        "Checking out into {:?} ...",
        prepare_checkout.repo().work_dir().expect("should be there")
    );

    let (repo, _) =
        prepare_checkout.main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;
    println!(
        "Repo cloned into {:?}",
        repo.work_dir().expect("directory pre-created")
    );

    let _remote = repo
        .find_default_remote(gix::remote::Direction::Fetch)
        .expect("always present after clone")?;

    Ok(())
}
