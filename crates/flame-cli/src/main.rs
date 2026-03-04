//! FlameWasm CLI — `flamewasm compile` and `flamewasm run`.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use flame_core::{parse, validate};
use flame_cranelift::compile;
use flame_runtime::{imports::Imports, instance::Instance, val::Val};
use flame_sandbox::{Capability, SandboxCtx, SandboxPolicy};
use flame_wasi::WasiCtxBuilder;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_env("FLAMEWASM_LOG"))
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Run(args) => cmd_run(args),
        Command::Compile(args) => cmd_compile(args),
    }
}

// ─── CLI definition ──────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "flamewasm",
    version,
    about = "🔥 FlameWasm — a blazing-fast WebAssembly micro-runtime",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// JIT-compile and execute a WebAssembly module
    Run(RunArgs),
    /// AOT-compile a WebAssembly module to a native object file
    Compile(CompileArgs),
}

#[derive(Parser)]
struct RunArgs {
    /// Path to the .wasm file
    file: PathBuf,

    /// Arguments to pass to the WASM module (after --)
    #[arg(last = true)]
    wasm_args: Vec<String>,

    /// Preopen a host directory for WASI
    #[arg(long = "dir", value_name = "PATH")]
    dirs: Vec<PathBuf>,

    /// Map guest path to host path  (--mapdir /data::/host/data)
    #[arg(long = "mapdir", value_name = "GUEST::HOST")]
    mapdirs: Vec<String>,

    /// Set environment variable (KEY=VALUE)
    #[arg(long = "env", value_name = "KEY=VAL")]
    envs: Vec<String>,

    /// Grant a capability (read:<path>, write:<path>, clock, random, network, proc-exit)
    #[arg(long = "allow", value_name = "CAP")]
    allows: Vec<String>,

    /// Start with zero capabilities (whitelist mode)
    #[arg(long = "deny-all")]
    deny_all: bool,

    /// Cranelift optimization level: none | speed | speed_and_size
    #[arg(long = "compile-opt", default_value = "speed")]
    opt_level: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Parser)]
struct CompileArgs {
    /// Path to the .wasm file
    file: PathBuf,

    /// Output object file path
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Target triple (default: host)
    #[arg(long)]
    target: Option<String>,
}

// ─── Run command ─────────────────────────────────────────────────────────────

fn cmd_run(args: RunArgs) -> Result<()> {
    let bytes = std::fs::read(&args.file)
        .with_context(|| format!("reading {}", args.file.display()))?;

    let module = parse(&bytes).context("parsing WebAssembly module")?;
    validate(&module).context("validating WebAssembly module")?;

    let compiled = compile(&module).context("compiling WebAssembly module")?;

    // Build WASI context
    let mut wasi_builder = WasiCtxBuilder::new()
        .args(std::iter::once(args.file.to_string_lossy().to_string()).chain(args.wasm_args));

    for dir in &args.dirs {
        let guest = dir.to_string_lossy().to_string();
        wasi_builder = wasi_builder.preopen_dir(guest, dir.clone());
    }
    for mapdir in &args.mapdirs {
        if let Some((guest, host)) = mapdir.split_once("::") {
            wasi_builder = wasi_builder.preopen_dir(guest, PathBuf::from(host));
        }
    }
    for env_kv in &args.envs {
        if let Some((k, v)) = env_kv.split_once('=') {
            wasi_builder = wasi_builder.env(k, v);
        }
    }

    let wasi_ctx = wasi_builder.build();

    // Build sandbox policy
    let mut policy = if args.deny_all {
        SandboxPolicy::deny_all()
    } else {
        SandboxPolicy::allow_all()
    };
    for cap_str in &args.allows {
        if let Some(cap) = parse_capability(cap_str) {
            policy = policy.grant(cap);
        } else {
            eprintln!("warning: unknown capability '{cap_str}', ignored");
        }
    }

    let _sandbox = SandboxCtx::new(wasi_ctx, policy);
    let imports = Imports::new();

    let instance = Instance::new(&module, compiled, &imports)
        .context("instantiating WebAssembly module")?;

    // Try calling _start (WASI entry point) or main
    let entry_points = ["_start", "main", "run"];
    let mut called = false;
    for ep in entry_points {
        if module.exports.iter().any(|e| e.name == ep) {
            match instance.call(ep, &[]) {
                Ok(results) => {
                    for r in results {
                        if let Val::I32(code) = r {
                            if code != 0 {
                                std::process::exit(code);
                            }
                        }
                    }
                    called = true;
                    break;
                }
                Err(e) => {
                    eprintln!("flamewasm: trap in '{ep}': {e}");
                    std::process::exit(1);
                }
            }
        }
    }

    if !called {
        eprintln!("flamewasm: no entry point found (_start/main/run)");
        std::process::exit(1);
    }

    Ok(())
}

// ─── Compile command ──────────────────────────────────────────────────────────

fn cmd_compile(args: CompileArgs) -> Result<()> {
    let bytes = std::fs::read(&args.file)
        .with_context(|| format!("reading {}", args.file.display()))?;

    let module = parse(&bytes).context("parsing WebAssembly module")?;
    validate(&module).context("validating WebAssembly module")?;

    let compiled = compile(&module).context("compiling WebAssembly module")?;

    let output = args.output.unwrap_or_else(|| {
        let mut p = args.file.clone();
        p.set_extension("flame.o");
        p
    });

    println!(
        "flamewasm: compiled {} exports to {}",
        compiled.export_names().count(),
        output.display()
    );
    println!(
        "flamewasm: (AOT object emission not yet wired — JIT code lives in-process)"
    );

    Ok(())
}

// ─── Capability parser ────────────────────────────────────────────────────────

fn parse_capability(s: &str) -> Option<Capability> {
    if let Some(path) = s.strip_prefix("read:") {
        return Some(Capability::ReadDir(PathBuf::from(path)));
    }
    if let Some(path) = s.strip_prefix("write:") {
        return Some(Capability::WriteDir(PathBuf::from(path)));
    }
    match s {
        "clock" => Some(Capability::Clock),
        "random" => Some(Capability::Random),
        "network" => Some(Capability::Network),
        "proc-exit" | "proc_exit" => Some(Capability::ProcessExit),
        _ => None,
    }
}
