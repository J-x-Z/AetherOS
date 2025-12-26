mod backend;

use backend::Backend;
#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
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

    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
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

        let mut frame_count = 0;
        
        while window.is_open() && !window.is_key_down(Key::Escape) {
            let fb_buffer = unsafe { backend.get_framebuffer(WIDTH, HEIGHT) };
            
            // Check Shift state
            let shift = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
            
            // Input Handling
            let keys = window.get_keys_pressed(minifb::KeyRepeat::Yes);
            for key in keys {
               let c = match key {
                   // Letters (with Shift = uppercase)
                   Key::A => Some(if shift { 'A' } else { 'a' }),
                   Key::B => Some(if shift { 'B' } else { 'b' }),
                   Key::C => Some(if shift { 'C' } else { 'c' }),
                   Key::D => Some(if shift { 'D' } else { 'd' }),
                   Key::E => Some(if shift { 'E' } else { 'e' }),
                   Key::F => Some(if shift { 'F' } else { 'f' }),
                   Key::G => Some(if shift { 'G' } else { 'g' }),
                   Key::H => Some(if shift { 'H' } else { 'h' }),
                   Key::I => Some(if shift { 'I' } else { 'i' }),
                   Key::J => Some(if shift { 'J' } else { 'j' }),
                   Key::K => Some(if shift { 'K' } else { 'k' }),
                   Key::L => Some(if shift { 'L' } else { 'l' }),
                   Key::M => Some(if shift { 'M' } else { 'm' }),
                   Key::N => Some(if shift { 'N' } else { 'n' }),
                   Key::O => Some(if shift { 'O' } else { 'o' }),
                   Key::P => Some(if shift { 'P' } else { 'p' }),
                   Key::Q => Some(if shift { 'Q' } else { 'q' }),
                   Key::R => Some(if shift { 'R' } else { 'r' }),
                   Key::S => Some(if shift { 'S' } else { 's' }),
                   Key::T => Some(if shift { 'T' } else { 't' }),
                   Key::U => Some(if shift { 'U' } else { 'u' }),
                   Key::V => Some(if shift { 'V' } else { 'v' }),
                   Key::W => Some(if shift { 'W' } else { 'w' }),
                   Key::X => Some(if shift { 'X' } else { 'x' }),
                   Key::Y => Some(if shift { 'Y' } else { 'y' }),
                   Key::Z => Some(if shift { 'Z' } else { 'z' }),
                   
                   // Numbers (with Shift = symbols)
                   Key::Key0 => Some(if shift { ')' } else { '0' }),
                   Key::Key1 => Some(if shift { '!' } else { '1' }),
                   Key::Key2 => Some(if shift { '@' } else { '2' }),
                   Key::Key3 => Some(if shift { '#' } else { '3' }),
                   Key::Key4 => Some(if shift { '$' } else { '4' }),
                   Key::Key5 => Some(if shift { '%' } else { '5' }),
                   Key::Key6 => Some(if shift { '^' } else { '6' }),
                   Key::Key7 => Some(if shift { '&' } else { '7' }),
                   Key::Key8 => Some(if shift { '*' } else { '8' }),
                   Key::Key9 => Some(if shift { '(' } else { '9' }),
                   
                   // Punctuation
                   Key::Period => Some(if shift { '>' } else { '.' }),
                   Key::Comma => Some(if shift { '<' } else { ',' }),
                   Key::Slash => Some(if shift { '?' } else { '/' }),
                   Key::Semicolon => Some(if shift { ':' } else { ';' }),
                   Key::Apostrophe => Some(if shift { '"' } else { '\'' }),
                   Key::LeftBracket => Some(if shift { '{' } else { '[' }),
                   Key::RightBracket => Some(if shift { '}' } else { ']' }),
                   Key::Backslash => Some(if shift { '|' } else { '\\' }),
                   Key::Minus => Some(if shift { '_' } else { '-' }),
                   Key::Equal => Some(if shift { '+' } else { '=' }),
                   Key::Backquote => Some(if shift { '~' } else { '`' }),
                   
                   // Special keys
                   Key::Space => Some(' '),
                   Key::Enter => Some('\n'),
                   Key::Backspace => Some('\x08'),
                   Key::Tab => Some('\t'),
                   
                   // Skip modifier keys themselves
                   Key::LeftShift | Key::RightShift => None,
                   Key::LeftCtrl | Key::RightCtrl => None,
                   Key::LeftAlt | Key::RightAlt => None,
                   
                   _ => None,
               };
               
               if let Some(char_val) = c {
                   backend.inject_key(char_val);
               }
            }
            
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
