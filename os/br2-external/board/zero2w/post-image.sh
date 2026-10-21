#!/bin/bash
# Post-image script: builds sdcard.img with genimage.
# $1 is the images directory, $2 the image variant (release or dev).
set -euo pipefail

BOARD_DIR="$(dirname "$0")"
VARIANT="${2:-release}"
GENIMAGE_CFG="${BINARIES_DIR}/genimage.cfg"
GENIMAGE_TMP="${BUILD_DIR}/genimage.tmp"

case "${VARIANT}" in
release)
    ROOTFS=rootfs.erofs
    EXTRA=""
    ;;
dev)
    ROOTFS=rootfs.ext4
    # template for the Wi-Fi settings, to be renamed wpa_supplicant.conf
    cat > "${BINARIES_DIR}/wpa_supplicant.conf.example" <<'WPA'
ctrl_interface=/run/wpa_supplicant
update_config=0
country=IT

network={
	ssid="network-name"
	psk="password"
}
WPA
    EXTRA=$'\n\t\t\t"wpa_supplicant.conf.example",'
    ;;
*)
    echo "post-image.sh: unknown variant '${VARIANT}'" >&2
    exit 1
    ;;
esac

sed -e "s|@ROOTFS@|${ROOTFS}|" \
    -e "s|@EXTRA_FILES@|${EXTRA//$'\n'/\\n}|" \
    "${BOARD_DIR}/genimage.cfg.in" > "${GENIMAGE_CFG}"

# genimage copies the root path it is given: pass an empty one, the root
# filesystem image is already built.
ROOTPATH_TMP="$(mktemp -d)"
trap 'rm -rf "${ROOTPATH_TMP}"' EXIT
rm -rf "${GENIMAGE_TMP}"

genimage \
    --rootpath "${ROOTPATH_TMP}" \
    --tmppath "${GENIMAGE_TMP}" \
    --inputpath "${BINARIES_DIR}" \
    --outputpath "${BINARIES_DIR}" \
    --config "${GENIMAGE_CFG}"
