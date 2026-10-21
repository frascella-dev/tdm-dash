# Architecture Decision Records

Each file records a project decision: the context, the choice and the consequences. A superseded decision is not deleted: a new ADR is written to replace it and the status of the old one is updated.

| # | Decision | Status |
|---|---|---|
| [0001](0001-pi-plus-microcontroller.md) | Pi Zero 2 W for graphics, RP2040 for I/O | Accepted |
| [0002](0002-usb-protocol.md) | USB CDC with postcard, COBS and CRC-16 with direction tag | Accepted |
| [0003](0003-slint-drm-platform.md) | Slint with its own DRM platform and software renderer | Accepted, to be confirmed by the benchmark |
| [0004](0004-odometer-in-fram.md) | Odometer on the microcontroller, in FRAM | Accepted |
| [0005](0005-read-only-rootfs.md) | Read-only EROFS rootfs, no orderly shutdown | Accepted |
| [0006](0006-fail-dark-backlight.md) | Backlight driven by the microcontroller, "fail-dark" | Accepted |
| [0007](0007-licenses.md) | Licenses of the code, the image, the hardware and the documents | Accepted |
| [0008](0008-toml-profiles.md) | Layered TOML bike profiles | Accepted |
| [0009](0009-seven-inch-display.md) | Newhaven 7" 800×480 1000 nit DPI display | Accepted |
| [0010](0010-default-layout.md) | Classic default layout | Accepted |
| [0011](0011-image-from-checkout.md) | The image builds the dashboard from the checkout (Buildroot `local` method) | Accepted |
| [0012](0012-english-repository-bilingual-ui.md) | English repository, bilingual dashboard | Accepted |
