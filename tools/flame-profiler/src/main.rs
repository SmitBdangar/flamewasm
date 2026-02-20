// flame-profiler: emit /tmp/perf-<pid>.map annotations for Linux perf / samply

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use flame_core::{parse, validate};
use flame_cranelift::compile;

#[derive(Parser)]
#[command(name = "flame-profiler", about = "Emit JIT perf map for a .wasm module")]
struct Cli {
    file: PathBuf,
    #[arg(long, default_value = "/tmp")]
    map_dir: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let bytes = std::fs::read(&cli.file)
        .with_context(|| format!("reading {}", cli.file.display()))?;
    let module = parse(&bytes).context("parse")?;
    validate(&module).context("validate")?;
    let compiled = compile(&module).context("compile")?;

    let pid = std::process::id();
    let map_path = cli.map_dir.join(format!("perf-{pid}.map"));
    let mut entries = String::new();

    for name in compiled.export_names() {
        if let Some(ptr) = compiled.get_export(name) {
            // Format: <start_addr> <size_hint> <symbol_name>
            entries.push_str(&format!("{:x} 100 flamewasm::{}\n", ptr as usize, name));
        }
    }

    std::fs::write(&map_path, &entries)
        .with_context(|| format!("writing {}", map_path.display()))?;

    println!("Wrote perf map: {}", map_path.display());
    println!("Run: perf record -p {pid} && perf report");
    Ok(())
}
