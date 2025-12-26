#![no_std]
#![no_main]

extern crate alloc;

use aether_user::{entry_point, console_println, console_init};
use wasmi::{Engine, Linker, Module, Store, Caller};

entry_point!(main);

fn main() -> ! {
    console_init();
    console_println("\n[AetherOS] Starting WASM Runtime...");
    
    // 1. Initialize WASM Engine
    let engine = Engine::default();
    let mut linker = <Linker<()>>::new(&engine);
    
    // 2. Define Host Function: env.print(ptr, len)
    linker.func_wrap("env", "print", |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {
        let mem = caller.get_export("memory").and_then(|e| e.into_memory()).unwrap();
        let mut buffer = alloc::vec![0u8; len as usize];
        
        // Read string from WASM memory
        if let Ok(_) = mem.read(&caller, ptr as usize, &mut buffer) {
            if let Ok(s) = core::str::from_utf8(&buffer) {
                // Print to AetherOS Console
                aether_user::console_println(s);
            }
        }
    }).unwrap();

    // 3. Load WASM Module (Hardcoded "Hello from WASM!" module)
    // WAT Source:
    // (module
    //   (type $t0 (func (param i32 i32)))
    //   (type $t1 (func))
    //   (import "env" "print" (func $print (type $t0)))
    //   (memory $memory 1)
    //   (export "memory" (memory $memory))
    //   (func $run (type $t1)
    //     i32.const 0
    //     i32.const 16
    //     call $print)
    //   (export "run" (func $run))
    //   (data (i32.const 0) "Hello from WASM!")
    // )
    let wasm_bytes: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
        0x01, 0x09, 0x02, 0x60, 0x02, 0x7f, 0x7f, 0x00, 0x60, 0x00, 0x00, // Types
        0x02, 0x0d, 0x01, 0x03, 0x65, 0x6e, 0x76, 0x05, 0x70, 0x72, 0x69, 0x6e, 0x74, 0x00, 0x00, // Import
        0x03, 0x02, 0x01, 0x01, // Function
        0x05, 0x03, 0x01, 0x00, 0x01, // Memory
        0x07, 0x11, 0x02, 0x06, 0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x01, // Exports
        0x0a, 0x0a, 0x01, 0x08, 0x00, 0x41, 0x00, 0x41, 0x10, 0x10, 0x00, 0x0b, // Code
        0x0b, 0x16, 0x01, 0x00, 0x41, 0x00, 0x0b, 0x10, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x66, 0x72, 0x6f, 0x6d, 0x20, 0x57, 0x41, 0x53, 0x4d, 0x21 // Data
    ];
    
    let module = Module::new(&engine, wasm_bytes).expect("Failed to parse WASM");
    let mut store = Store::new(&engine, ());
    
    // 4. Instantiate
    let instance = linker.instantiate(&mut store, &module).unwrap()
        .start(&mut store).unwrap();
        
    // 5. Call "run"
    let run = instance.get_typed_func::<(), ()>(&store, "run").unwrap();
    run.call(&mut store, ()).unwrap();
    
    console_println("[AetherOS] WASM Execution Finished.");
    
    loop {}
}
