################################################################################
#
# tdm-dash
#
################################################################################

# Built from the checkout this external tree belongs to, so an image always
# matches the commit it was made from (docs/adr/0011-image-from-checkout.md).
TDM_DASH_VERSION = $(shell git -C $(BR2_EXTERNAL_TDMDASH_PATH) describe --always --dirty 2>/dev/null || echo local)
TDM_DASH_SITE = $(realpath $(BR2_EXTERNAL_TDMDASH_PATH)/../..)
TDM_DASH_SITE_METHOD = local
TDM_DASH_LICENSE = MIT or Apache-2.0 (tdm-dash), GPL-3.0-only (binary, links Slint)
TDM_DASH_LICENSE_FILES = LICENSE-MIT LICENSE-APACHE

# Only the Rust workspace is needed to build: leave out the OS tree (with the
# Buildroot submodule and its output), the firmware and the documentation.
TDM_DASH_OVERRIDE_SRCDIR_RSYNC_EXCLUSIONS = \
	--exclude /target --exclude /os --exclude /firmware \
	--exclude /hardware --exclude /docs --exclude /design

TDM_DASH_PROFILE = $(call qstrip,$(BR2_PACKAGE_TDM_DASH_PROFILE))

# The release profile of the workspace (fat LTO, one codegen unit, abort on
# panic) is overridden by the cargo infrastructure: set it again.
TDM_DASH_CARGO_ENV = \
	CARGO_PROFILE_RELEASE_LTO=fat \
	CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
	CARGO_PROFILE_RELEASE_PANIC=abort \
	CARGO_TARGET_$(call UPPERCASE,$(RUSTC_TARGET_NAME))_RUSTFLAGS="-C target-cpu=cortex-a53"

# The local site method skips the download step and, with it, the
# `cargo vendor` post-process of the cargo infrastructure: vendor the
# dependencies here, with the same layout, before building offline.
define TDM_DASH_CARGO_VENDOR
	cd $(@D) && mkdir -p .cargo && \
		$(TARGET_MAKE_ENV) $(PKG_CARGO_ENV) \
		cargo vendor --locked VENDOR > .cargo/config.toml
endef
TDM_DASH_PRE_CONFIGURE_HOOKS += TDM_DASH_CARGO_VENDOR

define TDM_DASH_BUILD_CMDS
	cd $(@D) && $(TARGET_MAKE_ENV) $(TARGET_CONFIGURE_OPTS) \
		$(PKG_CARGO_ENV) $(TDM_DASH_CARGO_ENV) \
		cargo build --offline --release --locked \
			-p tdm-dash --no-default-features --features kms
	cd $(@D) && $(TARGET_MAKE_ENV) $(TARGET_CONFIGURE_OPTS) \
		$(PKG_CARGO_ENV) $(TDM_DASH_CARGO_ENV) \
		cargo build --offline --release --locked \
			-p tdm-link --bin tdmctl
endef

define TDM_DASH_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 0755 $(@D)/target/$(RUSTC_TARGET_NAME)/release/tdm-dash \
		$(TARGET_DIR)/usr/bin/tdm-dash
	$(INSTALL) -D -m 0755 $(@D)/target/$(RUSTC_TARGET_NAME)/release/tdmctl \
		$(TARGET_DIR)/usr/bin/tdmctl
	$(INSTALL) -D -m 0644 $(@D)/profiles/$(TDM_DASH_PROFILE).toml \
		$(TARGET_DIR)/etc/tdm-dash/default.toml
endef

# The same profile goes on the boot partition, where it can be edited from
# any PC (see board/zero2w/post-image.sh).
define TDM_DASH_INSTALL_IMAGES_CMDS
	$(INSTALL) -D -m 0644 $(@D)/profiles/$(TDM_DASH_PROFILE).toml \
		$(BINARIES_DIR)/tdm-dash.toml
endef
TDM_DASH_INSTALL_IMAGES = YES

$(eval $(cargo-package))
