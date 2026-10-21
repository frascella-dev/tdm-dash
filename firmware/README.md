# Sensor module firmware

Rust firmware ([embassy](https://embassy.dev)) for the Raspberry Pi Pico (RP2040) and the Pico 2 (RP2350). The application logic is `tdm_core::McuCore`, the same code the PC simulator runs; this crate only connects the hardware: pins, ADC, I²C, PWM, USB.

How to build, flash and check it: [manual, chapter 09](../docs/manual/09-software.md#pico-firmware).

```sh
cargo build --release                      # Pico
cargo build --release --target thumbv8m.main-none-eabihf --no-default-features --features rp2350   # Pico 2
python3 ../tools/elf2uf2.py target/thumbv6m-none-eabi/release/tdm-firmware tdm-firmware-rp2040.uf2
```
