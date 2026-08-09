# opkg-utils (vendored)

This directory contains a vendored copy of `opkg-build` from [seife/opkg-utils](https://github.com/seife/opkg-utils) (upstream helper scripts for the opkg package manager).

Ubuntu/WSL does not ship `opkg-utils` in default apt repositories, so the CIF IPK build uses this bundled script by default. If `opkg-build` is already on your PATH (for example from NI Linux RT or a Yocto SDK), the build prefers that installation.

Build host requirements:

- `binutils` (`ar` command)
- `tar`, `gzip`
- `python3` (for staging scripts)

```bash
sudo apt install binutils tar gzip python3
```
