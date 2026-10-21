# 0011 · The image builds the dashboard from the checkout

**Status:** accepted · October 2026

## Context

The Buildroot `tdm-dash` package must build the dashboard program. There are two ways:

- **Download it from a GitHub tag**, like normal packages. It needs a tag and its hash before every build, and the image may contain a program from a different commit than the `os/` files that build it.
- **Use the repository folder** that contains the external tree (Buildroot `local` method).

The operating system, the program and the profile live in the same repository and change together: a protocol change, for example, touches firmware, program and image in the same commit.

## Decision

- `TDM_DASH_SITE_METHOD = local`, with `TDM_DASH_SITE` equal to the repository root. The version is `git describe --always --dirty`.
- The sync excludes `os/` (Buildroot and its outputs), `firmware/`, `hardware/`, `docs/` and `design/`.
- The `local` method skips the download step and therefore the `cargo vendor` that Buildroot runs for cargo packages. The same command runs in a pre-configure hook, then the build runs `--offline --locked` as for the other cargo packages.
- The workspace build profile (fat LTO, a single codegen unit, `panic = abort`) is set again with environment variables, because Buildroot's cargo infrastructure overrides it.

## Consequences

- Every image matches exactly one commit, even during development and in pull request CI.
- `legal-info` does not archive the `tdm-dash` source (Buildroot does not do it for `local` packages). The source to publish for the GPL-3.0 is the repository at the release tag: the release notes say so.
- `cargo vendor` needs the network at build time. The dependencies stay cached in `os/dl/br-cargo-home`, which CI saves between builds.
