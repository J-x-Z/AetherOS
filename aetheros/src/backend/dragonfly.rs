use super::Backend;

pub struct DragonFlyBackend;

impl DragonFlyBackend {
    pub fn new() -> Self {
        panic!("DragonFly BSD Backend not implemented yet");
    }
}

impl Backend for DragonFlyBackend {
    
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
