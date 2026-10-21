# 0003 · Slint with its own DRM platform and software renderer

**Status:** accepted, to be confirmed by the benchmark on hardware (M4.0) · October 2026

## Context

The interface must be written in Rust, and it must be light and good-looking: anti-aliased needles and well-crafted fonts. Slint's `linuxkms` backend pulls in libinput, hence udev, and xkbcommon, at odds with a system without udev. The GPU of the Pi Zero 2 W (VideoCore IV) only offers GLES 2.0 through Mesa vc4.

## Decision

- **Slint 1.18**, with the **software renderer**.
- **Our own Slint platform**: `MinimalSoftwareWindow` plus two DRM "dumb" buffers with page-flip synchronized to vblank, using the `drm` crate.
- Before writing it, check whether `backend-linuxkms` builds without libinput; if so, use that.

The software renderer does not support transforms, so:
- the needles are `Path`s with the points computed in Rust (sin/cos)
- the dials are images generated at startup with tiny-skia from the profile
- no rotation or blur in the design

Plan B, only if the benchmark fails (p95 > 33 ms): FemtoVG renderer on Mesa vc4 GLES2.

## Consequences

- The binary depends only on glibc and cross-compiling is trivial.
- The app knows exactly when the first frame is on the screen, and only then turns on the backlight.
- The same code runs on the PC (winit + software renderer) with the same look.
- The distributed image is GPLv3 because of Slint (see ADR 0007).
