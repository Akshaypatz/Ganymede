# Ganymede – Build & Release Guide

This document covers how to build, package, and release Ganymede for **macOS** and **Windows**.

---

## Prerequisites

### All Platforms
```bash
# Install Node.js (18+)
# https://nodejs.org

# Install Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Install project dependencies
npm install
```

### macOS Only
```bash
# Xcode Command Line Tools (needed for linking)
xcode-select --install

# Apple Silicon: ensure arm64 target is present
rustup target add aarch64-apple-darwin

# Intel Mac: ensure x86_64 target is present
rustup target add x86_64-apple-darwin
```

### Windows Only
```powershell
# Install Visual Studio Build Tools (C++ workload)
# https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Or install via winget:
winget install Microsoft.VisualStudio.2022.BuildTools

# Install WebView2 (usually pre-installed on Windows 11)
# https://developer.microsoft.com/en-us/microsoft-edge/webview2/

# Tauri CLI (via npm — same as other platforms)
npm install -g @tauri-apps/cli
```

---

## Development (Hot Reload)

```bash
# macOS / Linux / Windows (same command)
npm run tauri dev
```

This starts the Vite dev server and the Tauri webview pointing at it. Rust code is recompiled on save.

---

## Production Build

### macOS – Universal Binary (Intel + Apple Silicon)
```bash
# Build for both architectures (fat binary / universal)
npm run tauri build -- --target universal-apple-darwin
```
Output: `src-tauri/target/universal-apple-darwin/release/bundle/dmg/Ganymede_*.dmg`

### macOS – Apple Silicon only
```bash
npm run tauri build -- --target aarch64-apple-darwin
```
Output: `src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/Ganymede_*.dmg`

### macOS – Intel only
```bash
npm run tauri build -- --target x86_64-apple-darwin
```
Output: `src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/Ganymede_*.dmg`

### Windows – x64
```powershell
# Run from PowerShell in the project root
npm run tauri build -- --target x86_64-pc-windows-msvc
```
Output: `src-tauri\target\x86_64-pc-windows-msvc\release\bundle\msi\Ganymede_*.msi`  
Also produces: `src-tauri\target\x86_64-pc-windows-msvc\release\bundle\nsis\Ganymede_*_x64-setup.exe`

### Windows – ARM64 (Surface Pro X, Copilot+ PCs)
```powershell
rustup target add aarch64-pc-windows-msvc
npm run tauri build -- --target aarch64-pc-windows-msvc
```
Output: `src-tauri\target\aarch64-pc-windows-msvc\release\bundle\msi\Ganymede_*_arm64_en-US.msi`

---

## Code Signing (macOS)

To distribute outside the Mac App Store without Gatekeeper warnings:

```bash
# 1. Export certificate from Keychain as Developer ID Application
# 2. Set env vars:
export APPLE_CERTIFICATE="base64-encoded-p12"
export APPLE_CERTIFICATE_PASSWORD="your-p12-password"
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
export APPLE_ID="your@apple.id"
export APPLE_PASSWORD="app-specific-password"  # NOT your real password
export APPLE_TEAM_ID="YOUR_TEAM_ID"

# 3. Build with notarization
npm run tauri build -- --target universal-apple-darwin
```

Tauri handles signing and notarization automatically when those environment variables are set.

---

## Code Signing (Windows)

```powershell
# Set env vars for EV or Standard certificate:
$env:TAURI_PRIVATE_KEY = "path\to\private.key"
$env:TAURI_KEY_PASSWORD = "your-key-password"

# Or use a PFX file:
$env:WINDOWS_CERTIFICATE = "base64-encoded-pfx"
$env:WINDOWS_CERTIFICATE_PASSWORD = "pfx-password"

npm run tauri build -- --target x86_64-pc-windows-msvc
```

---

## Running Rust Tests

```bash
cd src-tauri
cargo test -- --nocapture
```

Run a specific test:
```bash
cargo test test_create_project -- --nocapture
```

---

## Frontend Type Check

```bash
npx tsc --noEmit
```

---

## Updating Version

1. Edit `src-tauri/tauri.conf.json` → `version` field
2. Edit `src-tauri/Cargo.toml` → `version` field
3. Edit `package.json` → `version` field

All three must match for the build to succeed.

---

## Troubleshooting

### macOS: "Apple could not verify..."
Run: `xattr -cr /Applications/Ganymede.app` then re-open.

### Windows: Missing WebView2
Download and install from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

### Rust compile error: `openssl-sys`
On Linux/Windows, install OpenSSL dev headers. On macOS this is handled by the system. The project uses `rustls-tls` feature on `reqwest` to avoid this dependency.

### "error: failed to run custom build command for `tauri-build`"
Ensure Xcode CLT is installed (macOS) or Visual Studio Build Tools are installed (Windows).

## Cross-Compiling Windows EXE from macOS

> **Note:** Full Tauri installer creation (MSI/NSIS) for Windows **requires a Windows machine or CI runner** because the WiX toolset and NSIS are Windows-only.  
> However, you can **cross-compile the raw Windows executable** on macOS using `cross` (Docker-based), which produces an `.exe` you can then package manually or via CI.

### Option A – `cross` tool (Docker required)

```bash
# Install cross
cargo install cross --git https://github.com/cross-rs/cross

# Add the Windows GNU target
rustup target add x86_64-pc-windows-gnu

# Build the executable (not the full installer)
cd src-tauri
cross build --release --target x86_64-pc-windows-gnu

# The exe will be at:
# src-tauri/target/x86_64-pc-windows-gnu/release/ganymede.exe
```

Copy the result to `beta-preview/`:
```bash
cp src-tauri/target/x86_64-pc-windows-gnu/release/ganymede.exe beta-preview/
```

### Option B – GitHub Actions (recommended for installers)

Use the CI workflow in this file (see CI/CD section below) with a `windows-latest` runner to produce a full `.msi` or NSIS installer. This is the only supported way to create a signed Windows installer from a macOS developer machine.

### Option C – mingw-w64 on macOS (no Docker)

```bash
# Install mingw-w64 cross-toolchain via Homebrew
brew install mingw-w64

# Add the target
rustup target add x86_64-pc-windows-gnu

# Configure linker in ~/.cargo/config.toml
cat >> ~/.cargo/config.toml << 'EOF'
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
EOF

# Build the Rust binary (frontend must be built first)
cd /path/to/Ganymede
npm run build           # build Svelte frontend
cd src-tauri
cargo build --release --target x86_64-pc-windows-gnu

# Copy to beta-preview
cp target/x86_64-pc-windows-gnu/release/ganymede.exe ../beta-preview/
```

> **Limitation:** The exe built this way embeds the Tauri WebView but not the bundled frontend assets path. The full Tauri `tauri build` command handles asset embedding — for a distributable Windows app, use GitHub Actions.



---

## CI/CD (GitHub Actions Example)

```yaml
name: Build
on:
  push:
    tags: ['v*']

jobs:
  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: npm install
      - run: rustup target add aarch64-apple-darwin x86_64-apple-darwin
      - run: npm run tauri build -- --target universal-apple-darwin

  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: npm install
      - run: npm run tauri build -- --target x86_64-pc-windows-msvc
```
