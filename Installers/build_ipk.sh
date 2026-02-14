#!/bin/sh

 cd ./lv_zenoh_ipk

# Extract the version number from the control file
# This assumes the file is at ./control/control
PKG_VERSION=$(grep "^Version:" ./control/control | cut -d' ' -f2)

# Verify we actually got a version number
if [ -z "$PKG_VERSION" ]; then
    echo "Error: Could not find version in ./control/control"
    exit 1
fi

# Create the data tarball
pushd ./data/
tar --numeric-owner --group=0 --owner=0 -czf ../data.tar.gz ./*
popd

# Create the control tarball
pushd ./control/
tar --numeric-owner --group=0 --owner=0 -czf ../control.tar.gz ./*
popd

# Create the ipk using the variable in the filename
tar --numeric-owner --group=0 --owner=0 -cf ../lv-zenoh.${PKG_VERSION}.ipk ./debian-binary ./data.tar.gz ./control.tar.gz

rm ./data.tar.gz
rm ./control.tar.gz

echo "Successfully created ../lv-zenoh.${PKG_VERSION}.ipk"
