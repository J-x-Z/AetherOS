use super::Backend;

pub struct NetBsdBackend;

impl NetBsdBackend {
    pub fn new() -> Self {
        panic!("NetBSD Backend (nvmm) not implemented yet");
    }
}

impl Backend for NetBsdBackend {
    
    fn name(&self) -> &str {
        "NetBSD NVMM"
    }
    
    fn step(&self) -> super::ExitReason {
        unimplemented!();
    }
    
    unsafe fn get_framebuffer(&self, _width: usize, _height: usize) -> &[u32] {
        &[]
    }
}
