use clap::Clap;
use colored::Colorize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use tempfile::NamedTempFile;
use walkdir::WalkDir;

// TODO All of these shouldn't be Option's. One of these should be a required value
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
    test_path: Option<PathBuf>,
    // #[clap(long)]
    // files_to_test: Option<Vec<PathBuf>>,
}

// TODO maybe use a logging crate instead of doing... this
fn main() {
    let args = Opts::parse();
    // let files_to_test = args.files_to_test.unwrap();
    let files_to_test = walk(args.test_path.unwrap()).unwrap();
    dbg!(&files_to_test.len());
    let mut tests_run = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    for file in files_to_test {
        let frontmatter = extract_frontmatter(&file);
        match frontmatter {
            Some(f) => match get_serde_value(&f) {
                Ok(frontmatter_value) => {
                    if let Some(includes_value) = frontmatter_value.get("includes") {
                        match get_include_paths(
                            includes_value,
                            PathBuf::from("/home/humancalico/code/test262/harness"),
                        ) {
                            Ok(include_files) => {
                                match generate_final_file_to_test(&file, include_files) {
                                    Ok(file_to_run) => match run_file_in_node(file_to_run) {
                                        Ok(exit_status) => {
                                            if exit_status.success() {
                                                tests_run += 1;
                                                passed_tests += 1;
                                                println!("{} {:?}", "Great Success".green(), file);
                                            } else {
                                                tests_run += 1;
                                                failed_tests += 1;
                                                eprintln!("{} {:?}", "FAIL".red(), file);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to execute the file | Error: {:?}", e)
                                        }
                                    },
                                    Err(e) => eprintln!(
                                        "Couldn't generate file: {:?} to test | Err: {}",
                                        file, e
                                    ),
                                }
                            }
                            Err(e) => eprintln!("Not able to find {:?} | Err: {}", file, e),
                        }
                    }
                }
                Err(e) => eprintln!("Could not get serde value from frontmatter | Err: {}", e),
            },
            None => {
                match generate_final_file_to_test(&file, vec![]) {
                    Ok(file_to_run) => match run_file_in_node(file_to_run) {
                        Ok(exit_status) => {
                            if exit_status.success() {
                                tests_run += 1;
                                passed_tests += 1;
                                println!("{} {:?}", "Great Success".green(), file);
                            } else {
                                tests_run += 1;
                                failed_tests += 1;
                                eprintln!("{} {:?}", "FAIL".red(), file);
                            }
                        }
                        Err(e) => eprintln!("Failed to execute the file | Error: {:?}", e),
                    },
                    Err(e) => eprintln!("Couldn't generate file: {:?} to test | Err: {}", file, e),
                }
            }
        }
    }
    println!(
        "TOTAL: {}, FAILED: {}, PASSED: {}",
        tests_run.to_string().yellow(),
        failed_tests.to_string().red(),
        passed_tests.to_string().green()
    );
}

// TODO walk file in parallel using jwalk
fn walk(root_path: PathBuf) -> walkdir::Result<Vec<PathBuf>> {
    let mut final_paths: Vec<PathBuf> = vec![];
    for entry in WalkDir::new(root_path) {
        // FIXME possible unecessary clone
        let entry_clone = entry?.clone();
        if entry_clone.file_type().is_file() {
            final_paths.push(entry_clone.into_path());
        } else {
        }
    }
    Ok(final_paths)
}

// TODO use a Result here instead of an Option
// TODO do this using the FromStr trait maybe
fn extract_frontmatter(file_to_test: &PathBuf) -> Option<String> {
    // FIXME remove unwrap
    // TODO Read asynchronously
    let file_contents = fs::read_to_string(file_to_test).unwrap();
    // TODO cleanup using the and_then method
    let yaml_start = file_contents.find("/*---");
    if let Some(start) = yaml_start {
        let yaml_end = file_contents.find("---*/");
        if let Some(end) = yaml_end {
            // TODO remove unwrap here
            Some(file_contents.get(start + 5..end).unwrap().to_string())
        } else {
            eprintln!("This file has an invalid frontmatter");
            None
        }
    } else {
        eprintln!("frontmatter not found in file: {:?}", file_to_test);
        None
    }
}

fn get_serde_value(frontmatter_str: &str) -> serde_yaml::Result<serde_yaml::Value> {
    serde_yaml::from_str(frontmatter_str)
}

fn get_include_paths(
    includes_value: &serde_yaml::Value,
    include_path_root: PathBuf,
) -> serde_yaml::Result<Vec<PathBuf>> {
    let mut includes: Vec<String> = serde_yaml::from_value(includes_value.clone())?;
    let must_include = &mut vec!["assert.js".to_string(), "sta.js".to_string()];
    includes.append(must_include);
    let mut include_paths: Vec<PathBuf> = vec![];
    for include in includes {
        include_paths.push(include_path_root.join(include));
    }
    Ok(include_paths)
}

fn generate_final_file_to_test(
    file_to_test: &PathBuf,
    files_to_add: Vec<PathBuf>,
) -> std::io::Result<NamedTempFile> {
    // TODO do this asynchronously
    let mut contents = String::new();
    for file in files_to_add {
        let file_contents = fs::read_to_string(file)?;
        contents.push_str(&file_contents);
    }
    let file_to_test_contents = fs::read_to_string(file_to_test)?;
    contents.push_str(&file_to_test_contents);
    let mut file = tempfile::Builder::new().suffix(".js").tempfile()?;
    writeln!(file, "{}", contents)?;
    Ok(file)
}

fn run_file_in_node(file: NamedTempFile) -> std::io::Result<ExitStatus> {
    // TODO .status() waits for the command to execute
    // FIXME currently shows all the errors if node is not able to run the file
    Command::new("node").arg(file.path()).status()
}
