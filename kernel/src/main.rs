mod backend;

use backend::Backend;
#[cfg(not(target_os = "android"))]
use minifb::{Key, Window, WindowOptions};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Framebuffer Config
const WIDTH: usize = 640;
const HEIGHT: usize = 480;
// Guest Physical Address for FB: 0x20000 (128KB offset, allowing 64KB code + 64KB stack/data)
// WE NEED TO ENSURE MEMORY IS LARGE ENOUGH. 
// 640*480*4 = 1,228,800 bytes (~1.2MB).
// Let's allocate 4MB for the VM.

fn main() {
    println!("AetherOS Microkernel v0.3.0 (Graphics Enabled)");
    
    // We need shared access to the memory to draw it.
    // The Backend owns the memory. We need to restructure slightly.
    // Ideally, Backend exposes a method `get_framebuffer() -> &[u32]`.
    // But `hv_vm_map` uses raw pointers.
    // For simplicity: We will let the Backend run, and accessing the Safe Wrapper around the memory is hard if it's generic.
    // Let's make `MacBackend` share the raw pointer safely via a wrapper or Arc.
    
    // Hack for PoC: MacBackend init() blocks forever currently.
    // We need to change `init()` to `start()` and return control, or run in thread.
    
    // New Plan:
    // 1. Backend::new() -> returns instance.
    // 2. Instance has `get_ram_ptr()`.
    // 3. Thread spawn: instance.run().
    // 4. Main Loop: Read RAM via ptr, Update Window.

    let backend = Arc::new(backend::CurrentBackend::new());
    let backend_clone = backend.clone();
    
    thread::spawn(move || {
        backend_clone.run();
    });

    #[cfg(not(target_os = "android"))]
    {
        let mut window = Window::new(
            "AetherOS - Guest Display",
            WIDTH,
            HEIGHT,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

        // Limit to 60fps
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        while window.is_open() && !window.is_key_down(Key::Escape) {
            let fb_buffer = unsafe { backend.get_framebuffer(WIDTH, HEIGHT) };
            window.update_with_buffer(fb_buffer, WIDTH, HEIGHT).unwrap();
        }
    }
    
    #[cfg(target_os = "android")]
    {
        // Android Headless Loop (Logcat output only)
        loop { 
            thread::sleep(Duration::from_secs(1)); 
        }
    }
}
