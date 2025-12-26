use super::Backend;

pub struct OpenBsdBackend;

impl OpenBsdBackend {
    pub fn new() -> Self {
        panic!("OpenBSD Backend (vmm/pledge) not implemented yet");
    }
}

impl Backend for OpenBsdBackend {
    
    fn name(&self) -> &str {
        "OpenBSD vmm"
    }
    
    fn step(&self) -> super::ExitReason {
        unimplemented!();
    }
    
    unsafe fn get_framebuffer(&self, _width: usize, _height: usize) -> &[u32] {
        &[]
    }
}
