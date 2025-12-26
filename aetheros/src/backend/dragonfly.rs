use super::Backend;

pub struct DragonFlyBackend;

impl Backend for DragonFlyBackend {
    fn new() -> Self {
        panic!("DragonFlyBSD Backend not implemented yet");
    }
    
    fn name(&self) -> &str {
        "DragonFlyBSD VMM"
    }
    
    fn step(&self) -> super::ExitReason {
        unimplemented!();
    }
    
    unsafe fn get_framebuffer(&self, _width: usize, _height: usize) -> &[u32] {
        &[]
    }
}
