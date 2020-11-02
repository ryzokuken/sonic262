use clap::Clap;
use std::path::PathBuf;

#[derive(Clap)]
#[clap(
    name = "sonic262",
    version = "0.1.0",
    about = "A harness for test262",
    author = "Ujjwal Sharma <ryzokuken@disroot.org>"
)]
struct Opts {
    // #[clap(long)]
    // root_path: Option<PathBuf>,
    #[clap(long)]
    test_path: PathBuf,
    #[clap(long)]
    include_path: PathBuf,
    // #[clap(long)]
    // files_to_test: Option<Vec<PathBuf>>,
}

// TODO maybe use a logging crate instead of doing... this
fn main() {
    let args = Opts::parse();
    // let files_to_test = args.files_to_test.unwrap();
    let include_path = args.include_path;
    let test_path = args.test_path;
    // TODO remove unwrap here

    let diagnostics = sonic262::Diagnostics::default();

    sonic262::run_all(test_path, include_path, diagnostics)

}
