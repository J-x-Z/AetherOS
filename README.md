# AetherOS

<p align="center">
  <strong>🌐 The Universal Rust Platform</strong><br>
  <em>Run Rust applications anywhere — from macOS to Android, Windows to FreeBSD</em>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-1.70+-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/platforms-8+-blue" alt="Platforms">
  <img src="https://github.com/J-x-Z/AetherOS/actions/workflows/build.yml/badge.svg" alt="CI">
  <img src="https://img.shields.io/badge/license-GPLv3-blue" alt="License">
</p>

---

## What is AetherOS?

AetherOS is a **lightweight hypervisor-based microkernel** written in Rust. It allows you to run Rust `no_std` guest applications with **native performance** on any host operating system.

Think of it as: **Write once in Rust, run everywhere** — with direct hardware virtualization.

## ✨ Key Features

| Feature | Description |
|---------|-------------|
| 🖥️ **Graphics Support** | 640×480 framebuffer with direct pixel drawing |
| 🔧 **Universal ABI** | Hypercall-based communication between Guest and Host |
| 🌍 **8 Platform Backends** | macOS, Linux, Windows, Android, FreeBSD, NetBSD, OpenBSD, DragonFlyBSD |
| 🔒 **Memory Isolation** | Hardware-enforced VM separation via platform hypervisors |
| 📦 **Modular Architecture** | Rust workspace with separated kernel, ABI, and user libraries |

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Host OS (Any)                        │
├─────────────────────────────────────────────────────────┤
│  AetherOS Kernel (aether-kernel)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │ macOS       │  │ Linux/KVM   │  │ Windows/WHP     │  │
│  │ Hypervisor  │  │ Backend     │  │ Backend         │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │ FreeBSD     │  │ NetBSD      │  │ OpenBSD         │  │
│  │ bhyve       │  │ NVMM        │  │ vmm(4)          │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
├─────────────────────────────────────────────────────────┤
│  Guest VM (aarch64-unknown-none)                        │
│  ┌─────────────────────────────────────────────────┐    │
│  │ hello_world (Rust no_std application)           │    │
│  │ Uses: aether-user SDK                           │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites
- Rust toolchain (stable)
- `aarch64-unknown-none` target: `rustup target add aarch64-unknown-none`
- `cargo-binutils`: `cargo install cargo-binutils`

### Build & Run (macOS)

```bash
# 1. Build the guest application
cargo build --release -p hello_world --target aarch64-unknown-none
rust-objcopy -O binary target/aarch64-unknown-none/release/hello_world apps/hello_world/guest.bin

# 2. Build and sign the kernel
cargo build --release -p aether-kernel
codesign --entitlements kernel/entitlements.plist --force -s - target/release/aether-kernel

# 3. Run!
./target/release/aether-kernel
```

A window will appear displaying the guest's framebuffer output.

## 📁 Project Structure

```
AetherOS/
├── abi/                    # Shared ABI definitions (hypercall numbers)
├── apps/
│   └── hello_world/        # Example guest application
├── kernel/
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   └── backend/        # Platform-specific hypervisor implementations
│   │       ├── macos.rs    # Apple Hypervisor.framework
│   │       ├── linux.rs    # KVM (stub)
│   │       ├── windows.rs  # Windows Hypervisor Platform (stub)
│   │       ├── freebsd.rs  # bhyve (stub)
│   │       ├── netbsd.rs   # NVMM (stub)
│   │       ├── openbsd.rs  # vmm(4) (stub)
│   │       └── dragonfly.rs# DragonFlyBSD VMM (stub)
│   └── entitlements.plist  # macOS code signing entitlements
├── user/                   # Guest SDK (aether-user)
└── Cargo.toml              # Workspace definition
```

## 🖼️ Guest SDK (aether-user)

Write guest applications using the provided SDK:

```rust
#![no_std]
#![no_main]

use aether_user::{print, draw_pixel, fill_screen, SCREEN_WIDTH, SCREEN_HEIGHT};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("Hello from AetherOS Guest!");
    
    // Draw a gradient
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            let r = (x * 255 / SCREEN_WIDTH) as u8;
            let g = (y * 255 / SCREEN_HEIGHT) as u8;
            draw_pixel(x, y, r, g, 128);
        }
    }
    
    loop {}
}
```

## 🔮 Roadmap

See [ROADMAP.md](./ROADMAP.md) for detailed development plan.

### Phase 1: Core (Current)
- [x] Graphics Subsystem (Framebuffer)
- [x] macOS Backend (Hypervisor.framework)
- [x] Multi-platform CI (8 targets)
- [x] **TTY Console** - Text rendering on screen
- [x] **Input Handling** - Keyboard/Mouse events

### Phase 2: Backends
- [x] Linux KVM implementation
- [x] Windows WHP implementation
- [ ] BSD family implementations

### Phase 3: Ecosystem
- [ ] WASM Runtime integration
- [ ] Linux ABI compatibility layer
- [ ] Networking (VirtIO-net)

### Phase 4: Android
- [ ] DRM/KMS direct rendering
- [ ] SELinux policy & init.rc service

### Phase 5: Hybrid Kernel (Future)
- [ ] UEFI bootloader
- [ ] Bare metal kernel
- [ ] Hardware drivers


## 📚 Documentation

- [Android Architecture](./android_architecture.md) - Native hardware integration strategy
- [Ecosystem Bridge](./ecosystem_bridge.md) - Running existing software on AetherOS
- [WASM Integration](./wasm_integration_plan.md) - WebAssembly runtime plans

## 📄 License

GPLv3 License - See [LICENSE](./LICENSE) for details.

---

<p align="center">
  <em>Built with ❤️ and Rust</em>
</p>
