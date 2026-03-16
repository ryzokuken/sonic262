use std::io::Write;
use std::process::Command;
use std::time::Duration;

use wait_timeout::ChildExt;

use crate::types::RawResult;

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub host_path: String,
    pub host_args: Vec<String>,
    pub timeout_ms: u64,
}

pub fn run_single(code: &str, config: &RunConfig) -> Result<RawResult, std::io::Error> {
    let mut tmpfile = tempfile::NamedTempFile::new()?;
    tmpfile.write_all(code.as_bytes())?;
    tmpfile.flush()?;

    let mut cmd = Command::new(&config.host_path);
    for arg in &config.host_args {
        cmd.arg(arg);
    }
    cmd.arg(tmpfile.path());

    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let timeout = Duration::from_millis(config.timeout_ms);
    let status = child.wait_timeout(timeout)?;

    match status {
        Some(exit_status) => {
            let stdout = read_pipe(child.stdout.take());
            let stderr = read_pipe(child.stderr.take());
            Ok(RawResult {
                stdout,
                stderr,
                exit_code: exit_status.code(),
                timed_out: false,
            })
        }
        None => {
            child.kill()?;
            child.wait()?;
            let stdout = read_pipe(child.stdout.take());
            let stderr = read_pipe(child.stderr.take());
            Ok(RawResult {
                stdout,
                stderr,
                exit_code: None,
                timed_out: true,
            })
        }
    }
}

fn read_pipe(pipe: Option<impl std::io::Read>) -> String {
    match pipe {
        Some(mut p) => {
            let mut buf = String::new();
            let _ = std::io::Read::read_to_string(&mut p, &mut buf);
            buf
        }
        None => String::new(),
    }
}
