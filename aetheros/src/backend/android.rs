//! Android AVF Backend (Stub)
//! 
//! This backend is a placeholder for future AVF (Android Virtualization Framework) support.
//! Currently just allows compilation to pass.

use super::Backend;

pub struct AndroidBackend;

impl AndroidBackend {
    pub fn new() -> Self {
        println!("[Aether::AndroidBackend] AVF backend not yet implemented");
        AndroidBackend
    }
}

impl Backend for AndroidBackend {
    
    fn name(&self) -> &str {
        "Android AVF (Stub)"
    }
    
    fn step(&self) -> super::ExitReason {
        // No-op stub
        std::thread::sleep(std::time::Duration::from_secs(1));
        super::ExitReason::Yield
    }
    
    unsafe fn get_framebuffer(&self, _width: usize, _height: usize) -> &[u32] {
        // Return empty slice
        &[]
    }
}
