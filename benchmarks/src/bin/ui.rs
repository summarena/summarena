use anyhow::Result;
use clap::Parser;
use voyage_reranker_test::start_server;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Dataset name (e.g., fiqa, msmarco, scidocs)
    #[arg(short, long)]
    dataset: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    start_server(args.dataset).await
}