#!/bin/sh
# Build lv-zenoh .deb from Installers/lv_zenoh_deb using dpkg-deb.
#
# Stages under /tmp before packaging so DEBIAN permissions can be set correctly
# (dpkg-deb rejects 0777 on Windows/WSL drvfs mounts where chmod is ignored).
#
# Usage (from repo or Installers):
#   bash Installers/build_deb.sh
set -e

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
PKG_ROOT="${SCRIPT_DIR}/lv_zenoh_deb"

if [ ! -d "${PKG_ROOT}/DEBIAN" ]; then
  echo "Error: package root not found at ${PKG_ROOT}" >&2
  exit 1
fi

PKG_VERSION=$(grep "^Version:" "${PKG_ROOT}/DEBIAN/control" | cut -d' ' -f2)
if [ -z "${PKG_VERSION}" ]; then
  echo "Error: Could not find version in ${PKG_ROOT}/DEBIAN/control" >&2
  exit 1
fi

STAGING=$(mktemp -d)
cleanup() {
  rm -rf "${STAGING}"
}
trap cleanup EXIT

echo "Staging package tree..."
cp -a "${PKG_ROOT}/." "${STAGING}/"
cd "${STAGING}"

find . -name '.gitkeep' -type f -delete

chmod 755 DEBIAN
chmod 644 DEBIAN/control
for script in DEBIAN/preinst DEBIAN/postinst DEBIAN/prerm DEBIAN/postrm; do
  if [ -f "${script}" ]; then
    chmod 755 "${script}"
  fi
done

OUTPUT="${SCRIPT_DIR}/lv-zenoh.${PKG_VERSION}.deb"
echo "Building ${OUTPUT}..."
dpkg-deb --build --root-owner-group . "${OUTPUT}"

echo "Successfully created ${OUTPUT}"
