use predicates::prelude::*; // Used for writing assertions
use assert_cmd::Command;

use anyhow::Result;

#[test]
fn help_emits_usage() -> Result<()> {
    let mut cmd = Command::cargo_bin("piper")?;

    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("USAGE"));

    Ok(())
}

/// wants this to be running first
/// cargo run --release --package echoserver


#[test]
fn pipe_urls_success() -> Result<()> {
    let mut cmd = Command::cargo_bin("piper")?;

    // when 2 urls are piped in
    cmd.write_stdin("http://localhost:7878/ping?id=1\nhttp://localhost:7878/ping?id=2")
        .assert()
        .success()
        // then there are 2 pong responses back
        .stdout(predicate::str::is_match("(.*pong.*\n){2}")?);

    Ok(())
}

#[test]
fn url_template_success() -> Result<()> {
    let mut cmd = Command::cargo_bin("piper")?;

    cmd.args(&["-u", "http://localhost:7878/ping?id={1}"]);

    // when we send in a sequence of 5 numbers to a ping url
    cmd.write_stdin("1\n2\n3\n4\n5")
        .assert()
        .success()
        // then we will get five "pong" responses back
        .stdout(predicate::str::is_match("(.*pong.*\n){5}")?);

    Ok(())
}

