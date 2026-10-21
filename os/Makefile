# Builds the tdm-dash SD card images with Buildroot (docs/manuale/09-software.md).
#
#   make -C os release                    # image for the road: read-only EROFS
#   make -C os dev                        # development image: ext4, Wi-Fi, SSH
#   make -C os release-linux-menuconfig   # any Buildroot target, on either image
#   make -C os release-legal-info         # licences and sources to publish
#   make -C os clean
#
# Output in os/output/<image>/images/sdcard.img. Downloads and the compiler
# cache are shared by both images (os/dl, os/ccache).

OS_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
BR := $(OS_DIR)/buildroot
EXT := $(OS_DIR)/br2-external
OUT := $(OS_DIR)/output

export BR2_DL_DIR ?= $(OS_DIR)/dl
export BR2_CCACHE_DIR ?= $(OS_DIR)/ccache

DEFCONFIG_release := tdmdash_zero2w_defconfig
DEFCONFIG_dev := tdmdash_zero2w_dev_defconfig

.PHONY: all release dev clean
all: release

$(BR)/Makefile:
	git -C $(OS_DIR)/.. submodule update --init --depth 1 os/buildroot

# The configuration is regenerated whenever the defconfig changes.
define image
$(OUT)/$(1)/.config: $(EXT)/configs/$(DEFCONFIG_$(1)) | $(BR)/Makefile
	$$(MAKE) -C $(BR) O=$(OUT)/$(1) BR2_EXTERNAL=$(EXT) $(DEFCONFIG_$(1))

$(1): $(OUT)/$(1)/.config
	$$(MAKE) -C $(BR) O=$(OUT)/$(1)
	@echo "Immagine: $(OUT)/$(1)/images/sdcard.img"

$(1)-%: $(OUT)/$(1)/.config
	$$(MAKE) -C $(BR) O=$(OUT)/$(1) $$*
endef
$(eval $(call image,release))
$(eval $(call image,dev))

clean:
	rm -rf $(OUT)
