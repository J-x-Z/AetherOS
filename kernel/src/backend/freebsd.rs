use super::Backend;

pub struct FreeBsdBackend;

impl Backend for FreeBsdBackend {
    fn new() -> Self {
        panic!("FreeBSD Backend (bhyve) not implemented yet");
    }
    
    fn name(&self) -> &str {
        "bhyve"
    }
    
    fn run(&self) {
        unimplemented!();
    }
    
    unsafe fn get_framebuffer(&self, _width: usize, _height: usize) -> &[u32] {
        &[]
    }
}
