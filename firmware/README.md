# Firmware del modulo sensori

Firmware in Rust ([embassy](https://embassy.dev)) per il Raspberry Pi Pico (RP2040) e il Pico 2 (RP2350). La logica applicativa è `tdm_core::McuCore`, la stessa eseguita dal simulatore del PC; questo crate collega solo l'hardware: pin, ADC, I²C, PWM, USB.

Istruzioni per compilare, caricare e verificare: [manuale, capitolo 09](../docs/manuale/09-software.md#firmware-del-pico).

```sh
cargo build --release                      # Pico
cargo build --release --target thumbv8m.main-none-eabihf --no-default-features --features rp2350   # Pico 2
python3 ../tools/elf2uf2.py target/thumbv6m-none-eabi/release/tdm-firmware tdm-firmware-rp2040.uf2
```
