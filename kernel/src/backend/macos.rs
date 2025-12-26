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
const HV_REG_CPSR: u32 = 34;  // PSTATE/CPSR register
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
const RAM_SIZE: usize = 0x800000; // 8MB
const LOAD_ADDR: u64 = 0x0;  // Guest code at start of RAM (simpler layout)
const FB_ADDR: u64 = 0x100000; // Place Framebuffer at 1MB mark
const DISK_ADDR: usize = 0x300000; // Disk Image at 3MB mark

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
        println!("[Aether::MacBackend] Creating VM (8MB RAM)...");
        let mem;
        unsafe {
            // 1. Create VM
            let ret = hv_vm_create(std::ptr::null_mut());
            if ret != HV_SUCCESS {
                panic!("Failed to create VM: {}", ret);
            }

            // 2. Allocate 8MB aligned memory (64KB alignment for safety)
            let mem_layout = std::alloc::Layout::from_size_align(RAM_SIZE, 0x10000).unwrap();
            mem = std::alloc::alloc_zeroed(mem_layout) as *mut u8;          
            // 3. Map Memory - FLAT RWX
            let ret = hv_vm_map(mem as *const c_void, 0, RAM_SIZE, HV_MEMORY_READ | HV_MEMORY_WRITE | HV_MEMORY_EXEC);
             if ret != HV_SUCCESS {
                panic!("Failed to map memory: {}", ret);
            }
            
            // Load HVC Stub at 0x0
            let hvc_opcode: u32 = 0xd4000002;
            std::ptr::copy_nonoverlapping(&hvc_opcode as *const u32 as *const u8, mem, 4);
            
            // Load Guest Binary (Embedded)
            let guest_bin = include_bytes!("../../../apps/hello_world/guest-aarch64.bin");
            if guest_bin.len() > 0x80000 { panic!("Guest binary too large for Code Segment (max 512KB)!"); }
            std::ptr::copy_nonoverlapping(guest_bin.as_ptr(), mem, guest_bin.len());
            sys_icache_invalidate(mem as *const c_void, guest_bin.len());
            println!("[Aether::MacBackend] Loaded guest: {} bytes", guest_bin.len());

            // Load Disk Image (Embedded)
            // Note: In a real OS this would be mapped via virtio, but for now we RAM-load it.
            // We use standard file reading for disk.img to avoid recompiling kernel every time disk changes?
            // "include_bytes!" embeds it into kernel binary.
            // Let's try reading from file first if present (easier dev loop), else empty.
            if let Ok(disk_data) = std::fs::read("disk.img") {
                 let disk_ptr = mem.add(DISK_ADDR);
                 if disk_data.len() > (RAM_SIZE - DISK_ADDR) {
                     eprintln!("[Warning] Disk image too large for allocated RAM region!");
                 } else {
                     std::ptr::copy_nonoverlapping(disk_data.as_ptr(), disk_ptr, disk_data.len());
                     println!("[Aether::MacBackend] Loaded disk image: {} bytes at 0x{:x}", disk_data.len(), DISK_ADDR);
                 }
            } else {
                 eprintln!("[Aether::MacBackend] Warning: disk.img not found. Filesystem will be empty.");
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
            // Set CPSR to EL1h (0x3c4) - CRITICAL for HVC to trap to EL2!
            // Without this, vCPU runs in wrong exception level
            hv_vcpu_set_reg(vcpu, HV_REG_CPSR, 0x3c4);
            // Set SP to top of RAM (8MB - epsilon)
            hv_vcpu_set_sys_reg(vcpu, HV_SYS_REG_SP_EL1, 0x7FF000);
            hv_vcpu_set_sys_reg(vcpu, 0xc080, 0); // SCTLR_EL1 = 0 (MMU off)
            hv_vcpu_set_sys_reg(vcpu, 0xc082, 0x300000); // CPACR_EL1 = FPEN(11) -> Enable FP/SIMD
            
            // Debug: verify memory content at 0x0
            let first_instr = *(self.mem as *const u32);
            let second_instr = *(self.mem.add(4) as *const u32);
            println!("[Debug] Memory at 0x0: 0x{:08x} 0x{:08x}", first_instr, second_instr);
            
            let mut iter_count = 0u64;
            println!("[Debug] About to enter vCPU loop...");
            use std::io::Write;
            std::io::stdout().flush().unwrap();
            loop {
                hv_vcpu_run(vcpu);
                iter_count += 1;
                let reason = (*exit_info).reason;
                
                // Debug: print every exit
                // println!("[Debug] vCPU exit #{}, reason={}", iter_count, reason);
                if reason == HV_EXIT_REASON_EXCEPTION {
                    let syndrome = (*exit_info).exception.syndrome;
                    let ec = (syndrome >> 26) & 0x3F;
                    
                    if ec == 0x16 { // HVC
                        let mut x8: u64 = 0;
                        hv_vcpu_get_reg(vcpu, HV_REG_X8, &mut x8);
                        
                        if x8 == 0 { // Print
                           let mut gpa: u64 = 0; 
                           let mut len: u64 = 0;
                           hv_vcpu_get_reg(vcpu, HV_REG_X0, &mut gpa);
                           hv_vcpu_get_reg(vcpu, HV_REG_X1, &mut len);
                           
                           if gpa < RAM_SIZE as u64 && len < 1000 && len > 0 {
                               let ptr = self.mem.add(gpa as usize);
                               let slice = std::slice::from_raw_parts(ptr, len as usize);
                               let s = String::from_utf8_lossy(slice);
                               print!("[Guest] {}", s);
                               std::io::stdout().flush().unwrap();
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

    fn inject_key(&self, c: char) {
        unsafe {
            // MMIO Layout:
            // 0x80000: Status (1 = Data Ready, 0 = Empty)
            // 0x80004: Data (u32 ASCII)
            let status_ptr = self.mem.add(0x80000) as *mut u32;
            let data_ptr = self.mem.add(0x80004) as *mut u32;

            // Simple polling: only write if guest has consumed previous key (Status == 0)
            if std::ptr::read_volatile(status_ptr) == 0 {
                std::ptr::write_volatile(data_ptr, c as u32);
                std::ptr::write_volatile(status_ptr, 1); // Set ready flag
                // println!("[Kernel] Injected key: '{}'", c); // DEBUG
            } else {
                // println!("[Kernel] Key dropped (Guest Busy): '{}'", c); // DEBUG
            }
        }
    }
}


