#!/usr/bin/env python3
"""Convert the firmware ELF into a UF2 file for the Pico's USB bootloader.

Holding BOOTSEL while plugging in the Pico (or `tdmctl bootsel`) shows a
drive called RPI-RP2 / RP2350: copying the .uf2 onto it flashes the firmware.

    python3 tools/elf2uf2.py firmware/target/thumbv6m-none-eabi/release/tdm-firmware tdm-firmware.uf2
    python3 tools/elf2uf2.py --family rp2350 ELF OUT.uf2

Only standard-library code: no pyelftools, picotool or elf2uf2-rs needed.
"""

from __future__ import annotations

import argparse
import struct
import sys

UF2_MAGIC_START0 = 0x0A324655
UF2_MAGIC_START1 = 0x9E5D5157
UF2_MAGIC_END = 0x0AB16F30
UF2_FLAG_FAMILY_ID_PRESENT = 0x00002000
FAMILIES = {
    "rp2040": 0xE48BFF56,
    "rp2350": 0xE48BFF59,  # RP2350, ARM secure image
}
FLASH_START = 0x10000000
FLASH_END = 0x10000000 + 16 * 1024 * 1024
PAGE = 256
PT_LOAD = 1


def load_segments(elf: bytes) -> list[tuple[int, bytes]]:
    """Returns (physical address, data) for every loadable segment in flash."""
    if elf[:4] != b"\x7fELF" or elf[4] != 1 or elf[5] != 1:
        raise SystemExit("not a 32-bit little-endian ELF file")
    (e_phoff,) = struct.unpack_from("<I", elf, 28)
    e_phentsize, e_phnum = struct.unpack_from("<HH", elf, 42)
    segs = []
    for i in range(e_phnum):
        p_type, p_offset, _vaddr, p_paddr, p_filesz, _memsz, _flags, _align = struct.unpack_from(
            "<8I", elf, e_phoff + i * e_phentsize
        )
        if p_type != PT_LOAD or p_filesz == 0:
            continue
        if not (FLASH_START <= p_paddr < FLASH_END):
            continue  # RAM-only segment (.bss); .data is loaded via its LMA
        segs.append((p_paddr, elf[p_offset : p_offset + p_filesz]))
    if not segs:
        raise SystemExit("no loadable flash segments found")
    return segs


def to_pages(segs: list[tuple[int, bytes]]) -> dict[int, bytearray]:
    pages: dict[int, bytearray] = {}
    for addr, data in segs:
        for i, b in enumerate(data):
            a = addr + i
            page = a - (a % PAGE)
            pages.setdefault(page, bytearray(PAGE))[a - page] = b
    return pages


def uf2(pages: dict[int, bytearray], family: int) -> bytes:
    out = bytearray()
    addrs = sorted(pages)
    for n, addr in enumerate(addrs):
        block = struct.pack(
            "<8I",
            UF2_MAGIC_START0,
            UF2_MAGIC_START1,
            UF2_FLAG_FAMILY_ID_PRESENT,
            addr,
            PAGE,
            n,
            len(addrs),
            family,
        )
        block += bytes(pages[addr]) + bytes(476 - PAGE) + struct.pack("<I", UF2_MAGIC_END)
        assert len(block) == 512
        out += block
    return bytes(out)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("elf")
    ap.add_argument("out")
    ap.add_argument("--family", choices=sorted(FAMILIES), default="rp2040")
    a = ap.parse_args()
    with open(a.elf, "rb") as f:
        elf = f.read()
    pages = to_pages(load_segments(elf))
    data = uf2(pages, FAMILIES[a.family])
    with open(a.out, "wb") as f:
        f.write(data)
    print(f"{a.out}: {len(pages)} pagine da 256 byte ({len(pages) * PAGE // 1024} KiB in flash), famiglia {a.family}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
