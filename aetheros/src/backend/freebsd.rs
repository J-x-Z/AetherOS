use super::Backend;

pub struct FreeBsdBackend;

impl FreeBsdBackend {
    pub fn new() -> Self {
        panic!("FreeBSD Backend (bhyve) not implemented yet");
    }
}

impl Backend for FreeBsdBackend {
    
    fn name(&self) -> &str {
        "bhyve"
    }
    
    fn step(&self) -> super::ExitReason {
        unimplemented!();
    }
    
    unsafe fn get_framebuffer(&self, _width: usize, _height: usize) -> &[u32] {
        &[]
    }
}
