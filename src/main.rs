use std::path::PathBuf;

use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Ujjwal Sharma <ryzokuken@disroot.org>")]
struct Opts {
    #[clap(long)]
    root_path: Option<PathBuf>,
    #[clap(long)]
    test_path: Option<PathBuf>,
    #[clap(long)]
    include_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Opts::parse();
    let root_path = args.root_path;
    let test_path = args
        .test_path
        .unwrap_or_else(|| root_path.clone().unwrap().join("test"));
    let include_path = args
        .include_path
        .unwrap_or_else(|| root_path.unwrap().join("harness"));
    sonic262::run_test(test_path, include_path).await.unwrap();
}
