use bytes::Bytes;
use cargo_mirrorer::fetching::plan::FetchPlan;
use clap::Parser;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::TcpListener;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

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
    // #[arg(short = 'S', long = "serve", value_name = "git|sparse")]
    // pub serving: String,
    /// Crates that should be fetched for mirrorer to host
    #[arg(
        short = 'F',
        long = "fetch",
        value_name = "all|playground|n<number>|<comma separated list>"
    )]
    pub fetching: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger();

    let config = Config::parse();

    match FetchPlan::from_str(&config.fetching)? {
        FetchPlan::AllCrates => {
            // download_index_of_crates().await?;
            // download_crates_in_index().await?;
        }
        _ => todo!(),
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http2::Builder::new(LocalExec)
                .serve_connection(io, service_fn(echo))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
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

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
async fn echo(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(Response::new(full(
            "Try POSTing data to /echo such as: `curl localhost:3000/echo -XPOST -d \"hello world\"`",
        ))),

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[derive(Clone, Copy, Debug)]
struct LocalExec;

impl<F> hyper::rt::Executor<F> for LocalExec
where
    F: std::future::Future + 'static, // not requiring `Send`
{
    fn execute(&self, fut: F) {
        // This will spawn into the currently running `LocalSet`.
        tokio::task::spawn_local(fut);
    }
}
