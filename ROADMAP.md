# AetherOS Development Roadmap

## Phase 1: Core Functionality (Current → Q1 2025)

### 1.1 TTY / Text Console ⏳
- [ ] Embed 8x16 bitmap font (VGA-style) into `aether-user`
- [ ] Implement `Console` struct with cursor position and scrolling
- [ ] Rewrite `print!()` to render text directly to framebuffer
- [ ] Support basic ANSI escape codes (colors, cursor movement)

### 1.2 Input Handling ⏳
- [ ] Define input hypercalls (`AETHER_KEY_EVENT`, `AETHER_MOUSE_EVENT`)
- [ ] Implement keyboard event queue in kernel
- [ ] Forward `minifb` keyboard events to guest via shared memory
- [ ] Add `read_key()` and `poll_events()` to guest SDK

### 1.3 Hypercall Expansion ⏳
- [ ] `AETHER_GET_TIME` - System clock access
- [ ] `AETHER_SLEEP` - Guest sleep/yield
- [ ] `AETHER_ALLOC` - Dynamic memory allocation
- [ ] `AETHER_EXIT` - Clean guest termination

---

## Phase 2: Platform Backends (Q1-Q2 2025)

### 2.1 Linux KVM Backend ⏳
- [ ] Implement `/dev/kvm` IOCTLs in `linux.rs`
- [ ] Create VM, vCPU, and memory regions
- [ ] Handle VM exits and hypercall dispatch
- [ ] Test on x86_64 and aarch64 Linux

### 2.2 Windows WHP Backend ⏳
- [ ] Use `windows-rs` crate for WHP bindings
- [ ] Implement `WHvCreatePartition`, `WHvSetupPartition`
- [ ] Memory mapping and vCPU execution
- [ ] Test on Windows 11 with Hyper-V enabled

### 2.3 BSD Family Backends ⏳
- [ ] FreeBSD bhyve implementation
- [ ] NetBSD NVMM implementation
- [ ] OpenBSD vmm(4) implementation (requires nightly Rust)

---

## Phase 3: Ecosystem & Compatibility (Q2-Q3 2025)

### 3.1 WASM Runtime Integration ⏳
- [ ] Embed `wasmi` interpreter in kernel
- [ ] Define WASI-like imports for AetherOS
- [ ] Create `aether-wasm` guest app loader
- [ ] Support `wasm32-unknown-unknown` binaries

### 3.2 Linux ABI Shim ⏳
- [ ] ELF loader for Linux binaries
- [ ] Syscall translation layer (open, read, write, mmap)
- [ ] Basic `/proc` and `/dev/null` emulation
- [ ] Run simple CLI tools (busybox)

### 3.3 Networking ⏳
- [ ] VirtIO-net device emulation
- [ ] Simple TCP/IP stack in guest (`smoltcp`)
- [ ] `AETHER_NET_SEND` / `AETHER_NET_RECV` hypercalls

---

## Phase 4: Android Native (Q3-Q4 2025)

### 4.1 Direct Hardware Access ⏳
- [ ] DRM/KMS framebuffer backend (bypass minifb)
- [ ] `/dev/input/eventX` raw input handling
- [ ] SELinux policy module for AetherOS service

### 4.2 Android Integration ⏳
- [ ] Magisk module for easy installation
- [ ] `init.rc` service definition
- [ ] HIDL/AIDL GPU HAL client (if needed)
- [ ] Run as PID 1 replacement (advanced)

---

## Phase 5: Hybrid Kernel (2025+)

### 5.1 Bare Metal Bootstrap ⏳
- [ ] UEFI bootloader (`uefi-rs`)
- [ ] Minimal aarch64/x86_64 kernel entry
- [ ] Page table setup and exception handlers
- [ ] Serial console output (UART)

### 5.2 Hardware Abstraction ⏳
- [ ] VirtIO device drivers (GPU, Input, Net, Block)
- [ ] Timer and interrupt controller (GIC/APIC)
- [ ] Run in QEMU first, real hardware later

### 5.3 Driver Strategy ⏳
- [ ] Option A: Linux driver wrapper (like Fuchsia)
- [ ] Option B: VirtIO-only (QEMU/Cloud focus)
- [ ] Option C: Libhybris for Android blobs
- [ ] Decision deferred until Phase 4 complete

---

## Priority Order

| Priority | Phase | Effort | Impact |
|----------|-------|--------|--------|
| 🔴 HIGH | 1.1 TTY Console | 1 week | Enables debugging, demo-ready |
| 🔴 HIGH | 1.2 Input | 1 week | Interactive applications |
| 🟡 MED | 2.1 Linux KVM | 2 weeks | Server deployment |
| 🟡 MED | 3.1 WASM | 2 weeks | App ecosystem |
| 🟢 LOW | 4.x Android | 1 month | Mobile platform |
| 🔵 FUTURE | 5.x Hybrid Kernel | 3+ months | Full independence |

---

## Current Status

✅ Completed:
- Hypervisor.framework macOS backend
- 640x480 framebuffer graphics
- Universal ABI with hypercalls
- 8-platform CI matrix
- Cross-compilation for BSD family

⏳ In Progress:
- (Ready to start Phase 1.1)
