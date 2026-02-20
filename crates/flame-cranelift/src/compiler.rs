//! The compilation orchestrator: sets up Cranelift JIT and compiles each function.

use anyhow::{Context, Result};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{Linkage, Module as CraneliftModule};
use flame_core::ir::{ExportKind, ImportType, Module};
use tracing::debug;

use crate::{compiled_module::CompiledModule, func_translator::FuncTranslator};

/// AOT-compile a [`Module`] to native code via Cranelift JIT.
///
/// # Errors
/// Returns an error if any function fails to translate or the JIT engine fails.
pub fn compile(module: &Module) -> Result<CompiledModule> {
    let mut flag_builder = settings::builder();
    flag_builder.set("use_colocated_libcalls", "false").unwrap();
    flag_builder.set("is_pic", "false").unwrap();
    flag_builder.set("opt_level", "speed").unwrap();
    let isa_builder = cranelift_native::builder()
        .map_err(|e| anyhow::anyhow!("native ISA builder error: {e}"))?;
    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .map_err(|e| anyhow::anyhow!("ISA finish error: {e}"))?;

    let mut jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

    // Register imported functions as external symbols
    let imported_func_count =
        module.imports.iter().filter(|i| matches!(i.ty, ImportType::Func(_))).count();
    for import in &module.imports {
        if let ImportType::Func(_) = import.ty {
            let sym = format!("{}#{}", import.module, import.name);
            // Register with a null pointer placeholder; runtime will patch
            jit_builder.symbol(&sym, std::ptr::null_mut::<u8>());
        }
    }

    let mut jit_module = JITModule::new(jit_builder);
    let mut func_ids = Vec::new();

    // Declare imported functions
    let mut imported_sigs = Vec::new();
    for import in &module.imports {
        if let ImportType::Func(type_idx) = import.ty {
            let ft = module
                .types
                .get(type_idx as usize)
                .context("import type index OOB")?;
            let sig = crate::func_translator::build_signature(ft, &jit_module)?;
            let sym = format!("{}#{}", import.module, import.name);
            let fid = jit_module
                .declare_function(&sym, Linkage::Import, &sig)
                .context("declare imported function")?;
            func_ids.push(fid);
            imported_sigs.push(sig);
        }
    }

    // Declare local functions
    let local_func_start = func_ids.len();
    for (i, type_idx) in module.functions.iter().enumerate() {
        let ft = module
            .types
            .get(*type_idx as usize)
            .context("local func type index OOB")?;
        let sig = crate::func_translator::build_signature(ft, &jit_module)?;
        // Use export name if available, else synthesize
        let name = module
            .exports
            .iter()
            .find(|e| e.kind == ExportKind::Func && e.index as usize == imported_func_count + i)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| format!("__flame_func_{i}"));
        let fid = jit_module
            .declare_function(&name, Linkage::Export, &sig)
            .context("declare local function")?;
        func_ids.push(fid);
    }

    // Translate and define local functions
    for (i, type_idx) in module.functions.iter().enumerate() {
        let ft = module
            .types
            .get(*type_idx as usize)
            .context("local func type OOB")?;
        let body = module
            .code
            .get(i)
            .context("missing function body")?;
        let fid = func_ids[local_func_start + i];

        debug!("compiling function {i} (type_idx={type_idx})");

        let mut ctx = jit_module.make_context();
        let sig = crate::func_translator::build_signature(ft, &jit_module)?;
        ctx.func.signature = sig;

        let mut translator = FuncTranslator::new(
            module,
            ft,
            body,
            &func_ids,
            imported_func_count,
            &mut ctx.func,
            &mut jit_module,
        )?;
        translator.translate()?;

        jit_module
            .define_function(fid, &mut ctx)
            .context("define function")?;
        jit_module.clear_context(&mut ctx);
    }

    jit_module.finalize_definitions().context("finalize JIT definitions")?;

    // Collect exported function pointers
    let mut exports = std::collections::HashMap::new();
    for export in &module.exports {
        if export.kind == ExportKind::Func {
            let func_global_idx = export.index as usize;
            if func_global_idx < func_ids.len() {
                let fid = func_ids[func_global_idx];
                let ptr = jit_module.get_finalized_function(fid);
                exports.insert(export.name.clone(), ptr as usize);
            }
        }
    }

    Ok(CompiledModule {
        jit_module,
        func_ids,
        exports,
        imported_func_count,
    })
}
