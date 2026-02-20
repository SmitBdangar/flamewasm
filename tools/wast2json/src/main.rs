// wast2json: convert a .wast file's assert directives to JSON for CI tracking

use anyhow::Result;
use clap::Parser;
use serde_json::{json, Value};
use std::path::PathBuf;
use wast::{Wast, WastDirective};

#[derive(Parser)]
#[command(name = "wast2json", about = "Convert .wast spec file to JSON")]
struct Cli {
    file: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let src = std::fs::read_to_string(&cli.file)?;
    let buf = wast::parser::ParseBuffer::new(&src)?;
    let wast_file = wast::parser::parse::<Wast>(&buf)?;

    let mut asserts: Vec<Value> = Vec::new();
    for dir in wast_file.directives {
        let kind = match &dir {
            WastDirective::AssertReturn { .. } => "assert_return",
            WastDirective::AssertTrap { .. } => "assert_trap",
            WastDirective::AssertInvalid { .. } => "assert_invalid",
            WastDirective::AssertMalformed { .. } => "assert_malformed",
            _ => "other",
        };
        asserts.push(json!({ "kind": kind }));
    }

    let output = json!({
        "file": cli.file.to_string_lossy(),
        "directives": asserts.len(),
        "assertions": asserts,
    });
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
