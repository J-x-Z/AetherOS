# AetherOS

The Universal Rust Operating System.
"Write Once, Run Everywhere" (macOS, Linux, Android, Windows).

## Project Structure

### 1. `aether-kernel` (The Host)
The ultra-lightweight Hypervisor.
- **Backend-Agnostic**: Defines `trait Backend`.
- **macOS Implementation**: Uses `Hypervisor.framework` (Apple Silicon).
- **Goal**: Run `aether-abi` compatible binaries on any OS.

## 🌟 Current Capabilities (v0.3.0)

### 1. 🖥️ Graphics & UI
*   **Hardware Window**: Opens a native 640x480 window on the Host.
*   **Framebuffer**: Zero-copy shared memory graphics (Guest writes -> Host displays).
*   **API**: `draw_pixel` support in User SDK.

### 2. ⚡ Core Virtualization
*   **Hypervisor**: Native Apple Silicon support (`Hypervisor.framework`).
*   **Performance**: Raw hardware virtualization (EL1 execution).
*   **Memory**: 4MB RAM with flat RWX mapping (Stable boot).

### 3. 🔌 Universal ABI
*   **Hypercalls**:
    - `Print` (Debug logging).
    - `Exit` (System shutdown).
*   **Binary format**: Raw flat binary (`no_std`, layout agnostic).

### 4. 🧰 Developer Experience
*   **SDK**: `aether-user` crate treats low-level HVC as standard Rust functions.
*   **Monorepo**: Unified build system for Kernel, ABI, SDK, and Apps.

### 2. `aether-user` (The SDK)
The "Standard Library" for Aether apps.
- Hides the `HVC` assembly details.
- Provides `print!`, `exit`, `File`, `Net` (future).
- Usage: `use aether_user::{print, exit};`

### 3. `aether-abi` (The Interface)
Shared definitions ensuring binary compatibility between Kernel and User.

### 4. `apps/`
Example applications.
- `hello_world`: A minimal `no_std` Rust app demonstrating the stack.

## Getting Started

### Prerequisites
- Rust Nightly (for ASM features if needed, though mostly stable now).
- macOS (Apple Silicon) for the current Kernel verification.

### Build & Run
1. **Build the Workspace**:
   ```bash
   # Build the Kernel
   cargo build --release -p aether-kernel
   # Sign Kernel (Required on macOS)
   codesign --entitlements kernel/entitlements.plist --force -s - target/release/aether-kernel
   
   # Build the App
   cargo build --release -p hello_world --target aarch64-unknown-none
   # Extract Binary
   rust-objcopy -O binary target/aarch64-unknown-none/release/hello_world apps/hello_world/guest.bin
   ```

2. **Run**:
   ```bash
   cd kernel
   ../target/release/aether-kernel
   ```
