#!/bin/sh
# Post-build script: $1 is the target directory, $2 the image variant
# (release or dev, from BR2_ROOTFS_POST_SCRIPT_ARGS).
set -eu

TARGET_DIR="$1"
VARIANT="${2:-release}"

# mount point of the boot partition (profile, Wi-Fi settings)
mkdir -p "${TARGET_DIR}/boot"

if [ "${VARIANT}" = release ]; then
    # Everything the dashboard needs is built into the kernel and nothing
    # loads modules: keep them out of the read-only image.
    rm -rf "${TARGET_DIR}/lib/modules"
    # The rc scripts are never run (see /etc/inittab).
    rm -rf "${TARGET_DIR}/etc/init.d"
fi
