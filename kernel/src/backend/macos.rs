use super::Backend;
use std::ffi::c_void;
use std::mem;

// --- FFI Bindings for Hypervisor.framework ---
#[link(name = "Hypervisor", kind = "framework")]
unsafe extern "C" {
    fn hv_vm_create(config: *mut c_void) -> i32;
    fn hv_vm_destroy() -> i32;
    fn hv_vm_map(addr: *const c_void, ipa: u64, size: usize, flags: u64) -> i32;
    fn hv_vcpu_create(vcpu: *mut u64, exit: *mut *const HvVcpuExit, config: *mut c_void) -> i32;
    fn hv_vcpu_destroy(vcpu: u64) -> i32;
    fn hv_vcpu_run(vcpu: u64) -> i32;
    fn hv_vcpu_set_reg(vcpu: u64, reg: u32, value: u64) -> i32;
    fn hv_vcpu_get_reg(vcpu: u64, reg: u32, value: *mut u64) -> i32;
    fn hv_vcpu_set_sys_reg(vcpu: u64, reg: u16, value: u64) -> i32;
    fn hv_vcpu_get_sys_reg(vcpu: u64, reg: u16, value: *mut u64) -> i32;
    fn sys_icache_invalidate(start: *const c_void, len: usize);
}

// ARM64 Constants
const HV_SUCCESS: i32 = 0;
const HV_MEMORY_READ: u64 = 1 << 0;
const HV_MEMORY_WRITE: u64 = 1 << 1;
const HV_MEMORY_EXEC: u64 = 1 << 2;
const HV_REG_X0: u32 = 0;
const HV_REG_X1: u32 = 1;
const HV_REG_X8: u32 = 8;
const HV_REG_PC: u32 = 32;
const HV_SYS_REG_SP_EL1: u16 = 0xe208;
const HV_EXIT_REASON_EXCEPTION: u32 = 1;

#[repr(C)]
struct HvVcpuExit {
    reason: u32,
    exception: HvVcpuExitException,
}

#[repr(C)]
struct HvVcpuExitException {
    syndrome: u64,
    virtual_address: u64,
    physical_address: u64,
}

// Memory Layout
const RAM_SIZE: usize = 0x400000; // 4MB
const LOAD_ADDR: u64 = 0x10000;
const FB_ADDR: u64 = 0x100000; // Place Framebuffer at 1MB mark

pub struct MacBackend {
    mem: *mut u8, // Pointer to the start of allocated RAM
}

// Safety: We manage concurrent access carefully (Main thread reads FB, vCPU thread writes)
unsafe impl Send for MacBackend {}
unsafe impl Sync for MacBackend {}

impl Backend for MacBackend {
    fn name(&self) -> &str {
        "Hypervisor.framework (Apple Silicon)"
    }
    
    fn new() -> Self {
        println!("[Aether::MacBackend] Creating VM (4MB RAM)...");
        let mem;
        unsafe {
            // 1. Create VM
            let ret = hv_vm_create(std::ptr::null_mut());
            if ret != HV_SUCCESS {
                panic!("Failed to create VM: {}", ret);
            }

            // 2. Allocate 4MB aligned memory (64KB alignment for safety)
            let mem_layout = std::alloc::Layout::from_size_align(RAM_SIZE, 0x10000).unwrap();
            mem = std::alloc::alloc_zeroed(mem_layout);
            
            // 3. Map Memory - FLAT RWX (Debugging "Translation Fault")
            // Map the whole 4MB as Read/Write/Execute.
            // This ensures no holes and matches block alignment better.
            let ret = hv_vm_map(mem as *const c_void, 0, RAM_SIZE, HV_MEMORY_READ | HV_MEMORY_WRITE | HV_MEMORY_EXEC);
             if ret != HV_SUCCESS {
                panic!("Failed to map memory: {}", ret);
            }
            
            // Pointers for usage
            let code_ptr = mem.add(0x10000); // 0x10000 offset
            
            // Load HVC Stub at 0x0...


            // Load HVC Stub at 0x0
            let hvc_opcode: u32 = 0xd4000002;
            std::ptr::copy_nonoverlapping(&hvc_opcode as *const u32 as *const u8, mem, 4);
            
            // Load Guest Binary
            let bin_path = "apps/hello_world/guest.bin";
            if let Ok(code_data) = std::fs::read(bin_path) {
                if code_data.len() > 0x80000 { panic!("Guest binary too large for Code Segment (max 512KB)!"); }
                
                // Write code into Host memory (which backs the Guest RX region)
                std::ptr::copy_nonoverlapping(code_data.as_ptr(), code_ptr, code_data.len());
                sys_icache_invalidate(code_ptr as *const c_void, code_data.len());
                
                println!("[Aether::MacBackend] Loaded guest: {} bytes", code_data.len());
            } else {
                eprintln!("[Aether::MacBackend] Failed to load guest binary from {}", bin_path);
            }
        }
        
        MacBackend { mem }
    }

    fn run(&self) {
        println!("[Aether::MacBackend] Starting vCPU Loop...");
        unsafe {
            let mut vcpu: u64 = 0;
            let mut exit_info: *const HvVcpuExit = std::ptr::null();
            hv_vcpu_create(&mut vcpu, &mut exit_info, std::ptr::null_mut());
            
            // Init Registers
            hv_vcpu_set_reg(vcpu, HV_REG_PC, LOAD_ADDR);
            hv_vcpu_set_sys_reg(vcpu, HV_SYS_REG_SP_EL1, 0xFFFF); 
            hv_vcpu_set_reg(vcpu, 2, 0x3c5); // PSTATE
            hv_vcpu_set_sys_reg(vcpu, 0xc080, 0); // SCTLR_EL1 = 0
            
            loop {
                hv_vcpu_run(vcpu);
                let reason = (*exit_info).reason;
                
                if reason == HV_EXIT_REASON_EXCEPTION {
                    let ec = ((*exit_info).exception.syndrome >> 26) & 0x3F;
                    if ec == 0x16 { // HVC
                        let mut x8: u64 = 0;
                        hv_vcpu_get_reg(vcpu, HV_REG_X8, &mut x8);
                        
                        if x8 == 0 { // Print
                           let mut gpa: u64 = 0; 
                           let mut len: u64 = 0;
                           hv_vcpu_get_reg(vcpu, HV_REG_X0, &mut gpa);
                           hv_vcpu_get_reg(vcpu, HV_REG_X1, &mut len);
                           
                           if gpa < RAM_SIZE as u64 {
                               let ptr = self.mem.add(gpa as usize);
                               let slice = std::slice::from_raw_parts(ptr, len as usize);
                               let s = String::from_utf8_lossy(slice);
                               print!("[Guest] {}", s);
                           }
                           hv_vcpu_set_reg(vcpu, HV_REG_X0, 0);
                        } else if x8 == 1 { // Exit
                            println!("[Aether Guest] Exit.");
                            break;
                        }
                        
                        let mut pc: u64 = 0;
                        hv_vcpu_get_reg(vcpu, HV_REG_PC, &mut pc);
                        hv_vcpu_set_reg(vcpu, HV_REG_PC, pc + 4);
                    } else {
                        let mut pc: u64 = 0;
                        hv_vcpu_get_reg(vcpu, HV_REG_PC, &mut pc);
                        println!("[Kernel] Unhandled EC: 0x{:x}", ec);
                        println!("  PC: 0x{:x}", pc);
                        println!("  Syndrome: 0x{:x}", (*exit_info).exception.syndrome);
                        println!("  FAR: 0x{:x}", (*exit_info).exception.virtual_address);
                        break;
                    }
                } else {
                    break;
                }
            }
            hv_vcpu_destroy(vcpu);
        }
    }

    unsafe fn get_framebuffer(&self, width: usize, height: usize) -> &[u32] {
        let ptr = self.mem.add(FB_ADDR as usize) as *const u32;
        std::slice::from_raw_parts(ptr, width * height)
    }
}
