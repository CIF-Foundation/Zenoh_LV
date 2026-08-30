# Zenoh LabVIEW wrapper

This crate builds the native library LabVIEW loads for Zenoh (`zenoh_lv_wrapper`).
It is a Rust `cdylib` around the Zenoh 1.x API (locked to 1.7.2 in `Cargo.lock`).

You do **not** need a system install of Zenoh to compile. The crate is pulled from
crates.io. Install Zenoh itself only if you want a router or daemon for runtime testing:
https://zenoh.io/docs/getting-started/installation/

Run all `cargo` commands from this directory (`src/Wrapper`).

## Prerequisites

### All platforms

- [Rust](https://rustup.rs/) (stable). Edition 2021 is required.
- After installing rustup, confirm:

  ```bash
  rustc --version
  cargo --version
  ```

- First build downloads crates from crates.io (needs network). Later builds use `Cargo.lock`.

### Windows (64-bit DLL)

- Visual Studio 2022 with the **Desktop development with C++** workload.
- Open **x64 Native Tools Command Prompt for VS 2022** (not the x86 prompt, and not a
  plain `cmd` unless `cl.exe` is already on `PATH`). That environment provides the
  64-bit MSVC linker the `x86_64-pc-windows-msvc` toolchain needs.
- rustup default host should be `x86_64-pc-windows-msvc`.

### Linux (shared object)

- A C toolchain and common build deps, for example on Ubuntu/Debian/WSL:

  ```bash
  sudo apt update
  sudo apt install build-essential pkg-config libssl-dev
  ```

- Use rustup rather than distro `cargo` if the distro Rust is older than edition 2021 support.

## Build (Windows)

1. Open **x64 Native Tools Command Prompt for VS 2022**.
2. Change to this folder:

   ```bat
   cd /d <repo>\src\Wrapper
   ```

3. Release build (what the installers ship):

   ```bat
   cargo build --release
   ```

Output:

```text
target\release\zenoh_lv_wrapper.dll
target\release\zenoh_lv_wrapper.dll.lib
target\release\zenoh_lv_wrapper.pdb
```

Copy `zenoh_lv_wrapper.dll` to `Installers\Windows\resource\` before running
`Installers\Windows\build_nsis.bat`. The DLL is gitignored.

A debug build (`cargo build`) writes to `target\debug\` and is larger and slower;
use `--release` for anything you install on a target.

## Build (Linux / WSL)

From this directory:

```bash
cd src/Wrapper
cargo build --release
```

Output:

```text
target/release/libzenoh_lv_wrapper.so
```

That `.so` is for the OS/architecture you built on (typically `x86_64` glibc Linux).
Use it for the Ubuntu `.deb`. Copy it into:

```text
Installers/lv_zenoh_deb/usr/lib/cif/libzenoh_lv_wrapper.so
```

then run `bash Installers/build_deb.sh` from `Installers/` (see `Installers/readme.md`).

### LabVIEW RT (IPK)

The RT package architecture is `core2-64`. The `.so` inside the IPK must be built for
NI Linux RT, not a generic desktop Ubuntu library, unless you have confirmed they are ABI
compatible on your target.

Place `libzenoh_lv_wrapper.so` at:

```text
Installers/lv_zenoh_ipk/usr/lib/cif/libzenoh_lv_wrapper.so
```

or point the IPK build at a cargo output:

```bash
cd src/Wrapper && cargo build --release
SO_SOURCE=src/Wrapper/target/release/libzenoh_lv_wrapper.so bash Installers/build_lv_zenoh_ipk.sh
```

From Windows, `Installers\build_lv_zenoh_ipk.bat` runs that script through WSL. If the
`.so` was produced on Windows/WSL for desktop Linux, do not assume it will load on RT.

## Verify the artifact

Windows:

```bat
dir target\release\zenoh_lv_wrapper.dll
```

Linux:

```bash
ls -l target/release/libzenoh_lv_wrapper.so
file target/release/libzenoh_lv_wrapper.so
```

`file` should report a 64-bit shared object. On RT it should match the target (for
example x86-64, dynamically linked).

## Troubleshooting

- **`link.exe` / `unresolved external` on Windows:** the prompt is not the VS 2022
  **x64** Native Tools environment, or the C++ workload is missing.
- **`linker 'cc' not found` on Linux:** install `build-essential`.
- **Wrong bitness:** LabVIEW 64-bit needs a 64-bit DLL/`.so`. Do not use an x86
  developer prompt.
- **Stale dependency versions:** this repo checks in `Cargo.lock`. Prefer
  `cargo build --release` (uses the lockfile) over changing `zenoh` in `Cargo.toml`
  unless you intend to retarget the crate.
