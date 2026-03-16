use sonic262::parser::extract_frontmatter;

#[test]
fn extracts_basic_frontmatter() {
    let contents = r#"/*---
description: basic test
---*/
some code;
"#;
    let meta = extract_frontmatter(contents).unwrap();
    assert_eq!(meta.description, Some("basic test".to_string()));
}

#[test]
fn extracts_includes() {
    let contents = r#"/*---
includes: [propertyHelper.js, compareArray.js]
---*/
test();
"#;
    let meta = extract_frontmatter(contents).unwrap();
    assert_eq!(meta.includes, vec!["propertyHelper.js", "compareArray.js"]);
}

#[test]
fn extracts_flags() {
    let contents = r#"/*---
flags: [onlyStrict, async]
---*/
test();
"#;
    let meta = extract_frontmatter(contents).unwrap();
    assert!(meta.flags.contains(&sonic262::types::TestFlag::OnlyStrict));
    assert!(meta.flags.contains(&sonic262::types::TestFlag::Async));
}

#[test]
fn extracts_negative() {
    let contents = r#"/*---
negative:
  phase: parse
  type: SyntaxError
---*/
test();
"#;
    let meta = extract_frontmatter(contents).unwrap();
    let neg = meta.negative.unwrap();
    assert_eq!(neg.phase, sonic262::types::NegativePhase::Parse);
    assert_eq!(neg.error_type, "SyntaxError");
}

#[test]
fn extracts_features() {
    let contents = r#"/*---
features: [BigInt, Atomics]
---*/
test();
"#;
    let meta = extract_frontmatter(contents).unwrap();
    assert_eq!(meta.features, vec!["BigInt", "Atomics"]);
}

#[test]
fn no_frontmatter_returns_error() {
    let contents = "var x = 1;";
    assert!(extract_frontmatter(contents).is_err());
}

#[test]
fn extracts_no_strict_flag() {
    let contents = r#"/*---
flags: [noStrict]
---*/
test();
"#;
    let meta = extract_frontmatter(contents).unwrap();
    assert!(meta.flags.contains(&sonic262::types::TestFlag::NoStrict));
}

#[test]
fn extracts_module_flag() {
    let contents = r#"/*---
flags: [module]
---*/
export default 42;
"#;
    let meta = extract_frontmatter(contents).unwrap();
    assert!(meta.flags.contains(&sonic262::types::TestFlag::Module));
}

#[test]
fn extracts_raw_flag() {
    let contents = r#"/*---
flags: [raw]
---*/
'use strict'
[0]
's'.p = null;
"#;
    let meta = extract_frontmatter(contents).unwrap();
    assert!(meta.flags.contains(&sonic262::types::TestFlag::Raw));
}

#[test]
fn extracts_non_deterministic_flag() {
    let contents = r#"/*---
flags: [non-deterministic]
---*/
test();
"#;
    let meta = extract_frontmatter(contents).unwrap();
    assert!(meta.flags.contains(&sonic262::types::TestFlag::NonDeterministic));
}

#[test]
fn handles_crlf_line_endings() {
    let contents = "/*---\r\ndescription: crlf test\r\n---*/\r\ncode;\r\n";
    let meta = extract_frontmatter(contents).unwrap();
    assert_eq!(meta.description, Some("crlf test".to_string()));
}

#[test]
fn empty_frontmatter_returns_defaults() {
    let contents = r#"/*---
---*/
test();
"#;
    let meta = extract_frontmatter(contents).unwrap();
    assert!(meta.flags.is_empty());
    assert!(meta.features.is_empty());
    assert!(meta.includes.is_empty());
    assert!(meta.negative.is_none());
}
