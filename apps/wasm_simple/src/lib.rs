
// We don't need no_std for WASM target usually, but let's keep it simple.
// WASM environment provides basic things.

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Import print from host
extern "C" {
    fn print(ptr: *const u8, len: usize);
}

#[no_mangle]
pub extern "C" fn run() {
    let msg = "Hello from WASM!";
    unsafe {
        print(msg.as_ptr(), msg.len());
    }
}
