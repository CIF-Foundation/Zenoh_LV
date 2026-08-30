# Zenoh_LV
Provide support for Zenoh from LabVIEW.

**This project is in beta.** The API, packaging, and behavior may change.

Functions are provided for:

- Session initialize and finalize
- Subscribe and unsubscribe
- Read and write Double, U64, and I64 values
- Channel map accessors
- Node scanning

The project includes LabVIEW source, a Rust wrapper around the Zenoh API, VIPM packaging, and native-library installers for Windows, Linux, and Linux RT. Windows requires the DLL to be installed; Linux and Linux RT require the `.so` on the target. There is a separate installer for the native library (`lv_zenoh`).

## License

This project is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the full license text.
