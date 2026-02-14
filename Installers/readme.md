Instructions to build installers for RT (ipk) and Ubuntu (deb)
Update the version in both control files (ipk and deb)
In terminal run <wsl> to enter Windows Subsystem for Linux
Copy the installers directory to the home directory <cp -r  /mnt/d/dev/Packages/Zenoh_LV/Installers/ ~/installers>
Run the builds <bash ~/installers/build_deb.sh>  <bash ~/installers/build_ipk.sh>
Copy the files back to a mounted location <cp -r  ~/installers/lv-zenoh.0.1.0.0.ipk /mnt/d/dev/builds/installers/>
  <cp -r  ~/installers/lv-zenoh.0.1.0.0.deb /mnt/d/dev/builds/installers/>
Delete the new directory <rm -rf ~/installers>