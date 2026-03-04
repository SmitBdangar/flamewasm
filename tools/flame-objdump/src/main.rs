// flame-objdump: disassemble a compiled FlameWasm module's exports

use std::path::PathBuf;
use anyhow::{Context, Result};
use clap::Parser;
use flame_core::{parse, validate};
use flame_cranelift::compile;

#[derive(Parser)]
#[command(name = "flame-objdump", about = "Disassemble a FlameWasm .wasm module")]
struct Cli {
    file: PathBuf,
    #[arg(long, help = "Show export list")]
    exports: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let bytes = std::fs::read(&cli.file)
        .with_context(|| format!("reading {}", cli.file.display()))?;
    let module = parse(&bytes).context("parsing")?;
    validate(&module).context("validating")?;

    println!("Module: {}", cli.file.display());
    println!("  Types:     {}", module.types.len());
    println!("  Imports:   {}", module.imports.len());
    println!("  Functions: {}", module.functions.len());
    println!("  Tables:    {}", module.tables.len());
    println!("  Memories:  {}", module.memories.len());
    println!("  Globals:   {}", module.globals.len());
    println!("  Exports:   {}", module.exports.len());
    println!("  Code segs: {}", module.code.len());
    println!("  Data segs: {}", module.data.len());

    if cli.exports {
        println!("\nExports:");
        for exp in &module.exports {
            let _ty = module.types.get(
                module.functions.get(exp.index as usize).copied().unwrap_or(0) as usize
            );
            println!(
                "  [{:?}] {} (idx {})",
                exp.kind, exp.name, exp.index
            );
        }
    }

    println!("\nFunctions:");
    for (i, (type_idx, body)) in module.functions.iter().zip(module.code.iter()).enumerate() {
        let ft = &module.types[*type_idx as usize];
        println!(
            "  func[{}] type[{}] {} locals, {} instrs — {}",
            i, type_idx,
            body.locals.iter().map(|(c, _)| c).sum::<u32>(),
            body.instructions.len(),
            ft,
        );
    }

    let compiled = compile(&module).context("compile")?;
    println!("\nJIT exports:");
    for name in compiled.export_names() {
        if let Some(ptr) = compiled.get_export(name) {
            println!("  {} @ {:p}", name, ptr);
        }
    }

    Ok(())
}
