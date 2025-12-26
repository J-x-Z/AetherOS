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

use super::ExitReason;
use std::sync::Mutex;
use std::cell::UnsafeCell;

pub struct MacBackend {
    mem: *mut u8,
    // Hypervisor.framework checks thread affinity, so we verify or lazy init.
    // Wrapped in Mutex for Sync (Backend trait requires it), though we expect single scheduler thread for now.
    vcpu_state: Mutex<Option<(u64, *const HvVcpuExit)>>, 
}

// Safety: We manage concurrent access carefully
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

            // 2. Allocate 8MB aligned memory
            let mem_layout = std::alloc::Layout::from_size_align(RAM_SIZE, 0x10000).unwrap();
            mem = std::alloc::alloc_zeroed(mem_layout) as *mut u8;          
            // 3. Map Memory
            let ret = hv_vm_map(mem as *const c_void, 0, RAM_SIZE, HV_MEMORY_READ | HV_MEMORY_WRITE | HV_MEMORY_EXEC);
             if ret != HV_SUCCESS {
                panic!("Failed to map memory: {}", ret);
            }
            
            // Load HVC Stub
            let hvc_opcode: u32 = 0xd4000002;
            std::ptr::copy_nonoverlapping(&hvc_opcode as *const u32 as *const u8, mem, 4);
            
            // Load Guest Binary
            let guest_bin = include_bytes!("../../../apps/hello_world/guest-aarch64.bin");
            if guest_bin.len() > 0x80000 { panic!("Guest binary too large!"); }
            std::ptr::copy_nonoverlapping(guest_bin.as_ptr(), mem, guest_bin.len());
            sys_icache_invalidate(mem as *const c_void, guest_bin.len());
            println!("[Aether::MacBackend] Loaded guest: {} bytes", guest_bin.len());

            // Load local disk.img if present
            if let Ok(disk_data) = std::fs::read("disk.img") {
                 let disk_ptr = mem.add(DISK_ADDR);
                 if disk_data.len() > (RAM_SIZE - DISK_ADDR) {
                     eprintln!("[Warning] Disk image too large!");
                 } else {
                     std::ptr::copy_nonoverlapping(disk_data.as_ptr(), disk_ptr, disk_data.len());
                     println!("[Aether::MacBackend] Loaded disk image");
                 }
            }
        }
        
        MacBackend { 
            mem,
            vcpu_state: Mutex::new(None),
        }
    }

    fn step(&self) -> ExitReason {
        let mut state_guard = self.vcpu_state.lock().unwrap();
        
        let (vcpu, exit_info_ptr) = if let Some(state) = *state_guard {
            state
        } else {
            // Lazy Initialization on the Execution Thread
            println!("[Aether::MacBackend] Lazily Initializing vCPU on thread {:?}", std::thread::current().id());
            unsafe {
                let mut vcpu: u64 = 0;
                let mut exit_info: *const HvVcpuExit = std::ptr::null();
                // We pass null for exit info pointer pointer, wait hv_vcpu_create takes *mut *const
                let ret = hv_vcpu_create(&mut vcpu, &mut exit_info as *mut *const _, std::ptr::null_mut());
                if ret != 0 { panic!("Failed to create vCPU: {}", ret); }
                
                // Init Registers
                hv_vcpu_set_reg(vcpu, HV_REG_PC, LOAD_ADDR);
                hv_vcpu_set_reg(vcpu, HV_REG_CPSR, 0x3c4);
                hv_vcpu_set_sys_reg(vcpu, HV_SYS_REG_SP_EL1, 0x7FF000);
                hv_vcpu_set_sys_reg(vcpu, 0xc080, 0); // SCTLR_EL1 (MMU off)
                hv_vcpu_set_sys_reg(vcpu, 0xc082, 0x300000); // CPACR_EL1 (FPEN)
                
                *state_guard = Some((vcpu, exit_info));
                (vcpu, exit_info)
            }
        };

        unsafe {
            hv_vcpu_run(vcpu);
            let exit_info = &*exit_info_ptr;
            let reason = exit_info.reason;
            
            if reason == HV_EXIT_REASON_EXCEPTION {
                let syndrome = exit_info.exception.syndrome;
                let ec = (syndrome >> 26) & 0x3F;
                
                if ec == 0x16 { // HVC
                    // HVC Handling (Hypercalls)
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
                           use std::io::Write;
                           std::io::stdout().flush().unwrap();
                       }
                       hv_vcpu_set_reg(vcpu, HV_REG_X0, 0);
                    } else if x8 == 1 { // Exit
                        return ExitReason::Halt;
                    }
                    
                    // Advance PC
                    let mut pc: u64 = 0;
                    hv_vcpu_get_reg(vcpu, HV_REG_PC, &mut pc);
                    hv_vcpu_set_reg(vcpu, HV_REG_PC, pc + 4);
                    
                    return ExitReason::Yield; // Continue running
                } else {
                    println!("[Kernel] Unhandled EC: 0x{:x}", ec);
                    return ExitReason::Unknown;
                }
            } else {
                // Timer or other reasons would go here
                return ExitReason::Unknown;
            }
        }
    }

    unsafe fn get_framebuffer(&self, width: usize, height: usize) -> &[u32] {
        let ptr = self.mem.add(FB_ADDR as usize) as *const u32;
        std::slice::from_raw_parts(ptr, width * height)
    }

    fn inject_key(&self, c: char) {
        unsafe {
            let status_ptr = self.mem.add(0x80000) as *mut u32;
            let data_ptr = self.mem.add(0x80004) as *mut u32;

            if std::ptr::read_volatile(status_ptr) == 0 {
                std::ptr::write_volatile(data_ptr, c as u32);
                std::ptr::write_volatile(status_ptr, 1);
            }
        }
    }
}


