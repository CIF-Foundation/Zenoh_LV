Instructions to build installers for RT (ipk), Windows (NSIS), and Ubuntu (deb)

## lv_zenoh_ipk (LabVIEW RT)

1. Update the version in `Installers/lv_zenoh_ipk/CONTROL/control` when needed.
2. Place `libzenoh_lv_wrapper.so` in `Installers/lv_zenoh_ipk/usr/lib/cif/` before building (the `.so` is gitignored). Alternatively, build from Rust and set `SO_SOURCE`:

   `cd src/Wrapper && cargo build --release`

   `SO_SOURCE=src/Wrapper/target/release/libzenoh_lv_wrapper.so bash Installers/build_lv_zenoh_ipk.sh`

3. Build host needs `binutils`, `tar`, `gzip`, and `python3` (e.g. `sudo apt install binutils tar gzip python3`). The build uses the bundled `Installers/opkg-utils/opkg-build` script; Ubuntu/WSL does not provide an `opkg-utils` apt package.
4. From Windows, run:

   `Installers\build_lv_zenoh_ipk.bat`

   Or from WSL/Linux:

   `bash Installers/build_lv_zenoh_ipk.sh`

The build stages `Installers/lv_zenoh_ipk/`, normalizes text files to LF line endings, packages with `opkg-build`, and writes `lv-zenoh_<version>_<arch>.ipk` to `Installers/output/`.

`postinst` installs the library under `/usr/lib/cif/` and creates a symlink at `/usr/lib/libzenoh_lv_wrapper.so`.

## lv_zenoh Windows (NSIS)

1. Update the version in `Installers/Windows/lv-zenoh.nsi` when needed.
2. Run:

   `Installers\Windows\build_nsis.bat`

## lv_zenoh_deb (Ubuntu)

Update the version in `Installers/lv_zenoh_deb/DEBIAN/control`

From WSL/Linux:

`bash Installers/build_deb.sh`

The `.deb` is created in `Installers/`.
