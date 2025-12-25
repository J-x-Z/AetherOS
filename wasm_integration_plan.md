# WASM on AetherOS: Feasibility Study

You asked: "Is Application-level universality (WASM) really possible?"
The answer is **Yes**, but with specific caveats regarding GUI.

## 1. Why it works (The CPU Abstraction)
WebAssembly (WASM) effectively defines a "Virtual CPU".
- **Old Way**: You compile C code to `x86` or `ARM` machine code. It runs only on that chip.
- **WASM Way**: You compile C/Rust/Go to `wasm` bytecode.
- **AetherOS Role**: We include a small "Interpreter" (VM) or "JIT Compiler" in our Kernel.

## 2. Technical Implementation
In Rust, adding WASM support is surprisingly trivial because libraries like `wasmi` (Interpreter) or `wasmtime` (JIT) are `no_std` compatible!

### Proposed Kernel Code (`kernel/src/wasm.rs`)

```rust
// AetherOS can embed a WASM runtime directly
use wasmi::{Engine, Linker, Store, Module};

pub fn run_wasm_app(wasm_binary: &[u8]) {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm_binary).unwrap();
    let mut store = Store::new(&engine, ());
    
    // Link System Functions (ABI)
    let mut linker = Linker::new(&engine);
    linker.func_wrap("env", "aether_print", |caller: Caller<()>, ptr: i32, len: i32| {
        // Translate WASM memory read -> AetherOS print
    }).unwrap();
    
    // Start "main"
    let instance = linker.instantiate(&mut store, &module).unwrap().start(&mut store).unwrap();
    let main = instance.get_typed_func::<(), ()>(&mut store, "main").unwrap();
    main.call(&mut store, ()).unwrap();
}
```

## 3. The "Gotchas" (GUI & System APIs)
WASM handles math and logic perfectly. But what about Drawing Windows? Opening Files?
This is where **WASI (WebAssembly System Interface)** comes in, but we need to extend it.

- **Standard WASI**: Files, Clocks, Random Numbers. (AetherOS implements these headers).
- **GUI (The Hard Part)**: WASI doesn't define "Windows".
    - **Solution**: We define `aether_graphics.wasm` import module.
    - Apps call `import { draw_pixel } from 'aether_graphics'`.
    - AetherOS maps this directly to our Framebuffer logic.

## 4. Verdict
- **Computation**: 100% Solved. You can run `ffmpeg`, `grep`, `python` (WASM build) today.
- **GUI**: Requires custom bindings. Existing Linux GUI apps (GTK/Qt) **cannot** just magically run unless recompiles with a "Wasm Backend" (which Qt actually supports!).

So, it is valid for "New/Ported Apps", but it doesn't magically run existing binaries (that's what the Linux Shim is for).
