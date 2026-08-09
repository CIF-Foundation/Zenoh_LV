#!/usr/bin/env bash
# Create a lv-zenoh .ipk archive from a staged package directory using opkg-utils.
#
# opkg-build expects the standard opkg layout:
#   CONTROL/control (+ postinst, prerm, ...)
#   usr/, etc/, ...   (payload at package root, not under data/)
#
# Usage:
#   bash build_ipk.sh <package_directory> [output_directory]
#
# Requires opkg-build from opkg-utils. The build uses Installers/opkg-utils/opkg-build
# when opkg-build is not installed system-wide (Ubuntu/WSL does not ship opkg-utils).
# Also requires binutils (ar), tar, and gzip.
# The final IPK path is printed to stdout. Status messages go to stderr so
# callers can safely capture only the output filename.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUNDLED_OPKG_BUILD="${SCRIPT_DIR}/opkg-utils/opkg-build"

resolve_opkg_build() {
  if command -v opkg-build >/dev/null 2>&1; then
    command -v opkg-build
    return 0
  fi
  if [[ -f "${BUNDLED_OPKG_BUILD}" ]]; then
    echo "${BUNDLED_OPKG_BUILD}"
    return 0
  fi
  return 1
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 <package_directory> [output_directory]" >&2
  exit 1
fi

if ! OPKG_BUILD="$(resolve_opkg_build)"; then
  echo "Error: opkg-build not found. Expected bundled script at ${BUNDLED_OPKG_BUILD}" >&2
  exit 1
fi

if ! command -v ar >/dev/null 2>&1; then
  echo "Error: ar not found. Install binutils (e.g. apt install binutils)." >&2
  exit 1
fi

IPK_DIR="$(cd "$1" && pwd)"
OUTPUT_DIR="${2:-$(dirname "${IPK_DIR}")}"

resolve_control_dir() {
  if [[ -f "${IPK_DIR}/CONTROL/control" ]]; then
    echo "${IPK_DIR}/CONTROL"
  elif [[ -f "${IPK_DIR}/control/control" ]]; then
    echo "${IPK_DIR}/control"
  else
    return 1
  fi
}

if ! CONTROL_DIR="$(resolve_control_dir)"; then
  echo "Error: control file not found under CONTROL/ in ${IPK_DIR}" >&2
  exit 1
fi

# opkg-build only recognizes CONTROL/ or DEBIAN/ (case-sensitive on Linux).
if [[ "${CONTROL_DIR}" == "${IPK_DIR}/control" ]]; then
  mv "${IPK_DIR}/control" "${IPK_DIR}/CONTROL"
  CONTROL_DIR="${IPK_DIR}/CONTROL"
fi

# Package scripts must be executable for opkg-build validation.
for script in preinst postinst prerm postrm; do
  if [[ -f "${CONTROL_DIR}/${script}" ]]; then
    chmod 755 "${CONTROL_DIR}/${script}"
  fi
done

# Remove artifacts from older custom packaging, if present.
rm -f "${IPK_DIR}/debian-binary" "${IPK_DIR}/data.tar.gz" "${IPK_DIR}/control.tar.gz"

PKG="$(grep '^Package:' "${CONTROL_DIR}/control" | awk '{print $2}')"
VERSION="$(grep '^Version:' "${CONTROL_DIR}/control" | awk '{print $2}')"
ARCH="$(grep '^Architecture:' "${CONTROL_DIR}/control" | awk '{print $2}')"
if [[ -z "${PKG}" || -z "${VERSION}" || -z "${ARCH}" ]]; then
  echo "Error: Could not read Package, Version, and Architecture from ${CONTROL_DIR}/control" >&2
  exit 1
fi

mkdir -p "${OUTPUT_DIR}"
chmod +x "${OPKG_BUILD}"
"${OPKG_BUILD}" -o 0 -g 0 "${IPK_DIR}" "${OUTPUT_DIR}" >&2

IPK_FILENAME="${OUTPUT_DIR}/${PKG}_${VERSION}_${ARCH}.ipk"
if [[ ! -f "${IPK_FILENAME}" ]]; then
  echo "Error: Expected IPK not found at ${IPK_FILENAME}" >&2
  exit 1
fi

echo "Successfully created ${IPK_FILENAME}" >&2
printf '%s\n' "${IPK_FILENAME}"
