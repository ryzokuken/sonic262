use sonic262::runner::{run_single, RunConfig};

fn config() -> RunConfig {
    RunConfig {
        host_path: "node".to_string(),
        host_args: Vec::new(),
        timeout_ms: 10_000,
    }
}

#[test]
fn runs_passing_script() {
    let code = r#"console.log("hello");"#;
    let result = run_single(code, &config()).unwrap();
    assert_eq!(result.exit_code, Some(0));
    assert!(result.stdout.contains("hello"));
    assert!(!result.timed_out);
}

#[test]
fn captures_runtime_error() {
    let code = r#"throw new TypeError("bad");"#;
    let result = run_single(code, &config()).unwrap();
    assert_ne!(result.exit_code, Some(0));
    assert!(result.stderr.contains("TypeError"));
}

#[test]
fn captures_syntax_error() {
    let code = r#"var ;"#;
    let result = run_single(code, &config()).unwrap();
    assert_ne!(result.exit_code, Some(0));
    assert!(result.stderr.contains("SyntaxError"));
}

#[test]
fn times_out_infinite_loop() {
    let code = r#"while(true) {}"#;
    let cfg = RunConfig {
        timeout_ms: 500,
        ..config()
    };
    let result = run_single(code, &cfg).unwrap();
    assert!(result.timed_out);
}
