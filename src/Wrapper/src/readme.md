To compile new shared object on Linux:
https://zenoh.io/docs/getting-started/installation/
Install the C++ support package for Zenoh (deb or ipk)
Build with the command:
g++ -shared -fPIC -std=c++17 -fvisibility=default zenoh_wrapper.cpp -o libzenoh_wrapper.so -L/usr/local/lib -lzenohc

- shared: Required to create a shared library (.so) instead of an executable.
- fPIC: Generates Position Independent Code, which is mandatory for shared libraries so they can be loaded at any memory address.
- std=c++17: Ensures compatibility with the zenoh-cpp headers.
- fvisibility=default: Ensures that functions marked with your DLLEXPORT macro are visible to the LabVIEW loader.
- L/usr/local/lib -lzenohc: Tells the compiler where to find the Zenoh C library and links against it. 



To compile a new dll on Windows:
Download and unzip the Zenoh C++ for Windows.  (https://github.com/eclipse-zenoh/zenoh-c/releases) - zenoh-c-1.7.2-x86_64-pc-windows-msvc-standalone.zip
In this example it was unzipped at D:\dev\zenoh\c
Open x64 Native Tools Command Prompt for VS 2022
Navigate to this directory
Build with the command:
cl.exe /MD /LD /I"D:\dev\zenoh\c\include" zenoh_wrapper.cpp /link /DLL /LIBPATH:"D:\dev\zenoh\c\lib" zenohc.lib Advapi32.lib Iphlpapi.lib Ws2_32.lib /NODEFAULTLIB:libcmt.lib /OUT:zenoh_wrapper.dll

Where 
"D:\dev\zenoh\c\include" is the path to include directory downloaded from the C++ support package
"D:\dev\zenoh\c\lib" is the path to lib directory downloaded from the C++ support package

- /MD Multi-threaded dll flag
- /I"D:\dev\zenoh\include": Points to the folder containing the Zenoh header files (e.g., zenoh.h).
- /LIBPATH:"D:\dev\zenoh\wrapper": Points to the folder where zenohc.lib is located.
- zenohc.lib: The import library for your dependency. Even if you are using the DLL, the compiler needs this .lib file to resolve function addresses at build time.
- Advapi32.lib: Provides SystemFunction036 (also known as RtlGenRandom), which Zenoh uses for secure random number generation.
- Iphlpapi.lib: Provides GetAdaptersAddresses, which Zenoh uses to discover network interfaces for scouting.
- Ws2_32.lib: The standard Windows Sockets library required for any network-based C++ application. 
- /OUT:zenoh_wrapper.dll: The name of your newly created 64-bit DLL. 

Critical Requirements:
Environment: You must use the "x64 Native Tools Command Prompt" to ensure cl.exe targets 64-bit architecture.
Runtime Dependency: For your new DLL to load successfully later, zenohc.dll must be in the same folder as zenoh_wrapper.dll or in your system's PATH.
Exports: Ensure your functions in zenoh_wrapper.cpp are prefixed with __declspec(dllexport) so they are accessible once compiled. 