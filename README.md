FlameWasm
=========

A fast WebAssembly micro-runtime with ahead-of-time compilation, WASI
preview-1, and capability-based sandboxing — written in pure Rust.

Passes 98.4% of the official WebAssembly spec test suite.


BUILDING
--------

    git clone https://github.com/SmitBdangar/flamewasm.git
    cd flamewasm
    cargo build --workspace
    cargo test --workspace
    cargo run -p flame-spec-tests -- --report   # spec compliance report


INSTALL
-------

    cargo install --path crates/flame-cli


USAGE
-----

    # Run a WASI module
    flamewasm run hello.wasm

    # Preopen a host directory
    flamewasm run app.wasm --dir /tmp/data

    # Whitelist-mode sandbox: deny everything, grant only what's needed
    flamewasm run app.wasm --deny-all --allow read:/var/data --allow clock

    # AOT-compile to a native object
    flamewasm compile app.wasm -o app.flame.o


SANDBOXING
----------

Capabilities are denied by default. Grant them explicitly:

    flamewasm run app.wasm \
      --deny-all \
      --allow read:/var/data \
      --allow write:/tmp/output \
      --allow clock \
      --allow random

Programmatic API:

    use flame_sandbox::{SandboxPolicy, Capability, SandboxCtx};
    use flame_wasi::WasiCtx;

    let policy = SandboxPolicy::deny_all()
        .grant(Capability::ReadDir("/var/data".into()))
        .grant(Capability::WriteDir("/tmp/output".into()))
        .grant(Capability::Clock);

    let ctx = SandboxCtx::new(WasiCtx::builder().build(), policy);


CLI REFERENCE
-------------

    flamewasm run [OPTIONS] <FILE> [-- ARGS...]

      --dir <PATH>           Preopen a host directory (WASI)
      --mapdir <G>::<H>      Map guest path G to host path H
      --env <KEY=VAL>        Set an environment variable
      --allow <CAP>          Grant capability: read:<path>, write:<path>,
                             clock, random, network
      --deny-all             Start with zero capabilities
      --compile-opt <LEVEL>  none | speed | speed_and_size  [default: speed]
      -v, --verbose          Verbose logging

    flamewasm compile [OPTIONS] <FILE>

      -o, --output <FILE>    Output object file  [default: <input>.flame.o]
      --target <TRIPLE>      Target triple       [default: host]


CRATES
------

  flame-core         WASM binary parser, IR structs, spec-compliant validator
  flame-cranelift    Cranelift AOT compiler; IR -> native code
  flame-runtime      Instance lifecycle, linear memory, tables, trap handling
  flame-wasi         WASI preview-1 host functions
  flame-sandbox      Capability tokens and policy enforcement
  flame-cli          CLI frontend (compile + run)
  flame-spec-tests   Official spec test runner


SPEC TEST RESULTS
-----------------

  Proposal          Pass    Fail   Rate
  --------          ----    ----   ----
  MVP               4120      53   98.7%
  Memory             312       2   99.4%
  Control Flow       680       8   98.8%
  SIMD (partial)    1240      42   96.7%
  Total             6352     105   98.4%


LICENSE
-------

MIT OR Apache-2.0, at your option. See LICENSE-MIT and LICENSE-APACHE.
