To compile new shared object on Linux:
https://zenoh.io/docs/getting-started/installation/
Build with the command:
cargo build --release

To compile a new dll on Windows:
Open x64 Native Tools Command Prompt for VS 2022
Navigate to this directory
Build with the command:
cargo build --release

Note:
Environment: Using "x64 Native Tools Command Prompt" ensures the correct C++ 64 bit build tools are linked.