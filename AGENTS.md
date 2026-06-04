# Agent Instructions

## Project Overview

`backup-checker` is a Rust CLI and library for comparing two directory trees by file checksums and reporting files from the old directory that are missing in the new directory. The CLI entry point is `src/main.rs`; reusable logic lives in `src/lib.rs`.

## Repository Layout

- `src/lib.rs`: recursive file discovery, checksum generation, and missing-file detection.
- `src/main.rs`: CLI argument parsing and console output.
- `src/banner.rs`: CLI banner rendering.
- `Cargo.toml`: crate metadata, binary/library targets, dependencies, and optional checksum features.
- `pkgbuild/`: Arch Linux packaging files.
- `.github/workflows/build_rust.yml`: release builds for Linux, Windows, and macOS.

## Build And Test Commands

Run these from the repository root:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo build --release`

Use `cargo build --release --no-default-features` when checking the minimal CRC32-only build. The README uses `cargo build -r`, which is equivalent to a release build on supported Cargo versions.

## Coding Guidelines

- Keep the public library API simple and deterministic; avoid changing function signatures unless the caller impact is intentional.
- Prefer `Path`/`PathBuf`-oriented APIs for filesystem work. Avoid adding more `String` path handling unless needed for compatibility with existing public functions.
- Treat filesystem errors deliberately. Existing code uses `unwrap()`, but new code should prefer returning or handling errors where practical.
- Preserve feature-gated checksum support. Optional algorithms are controlled by the `sha256`, `adler32`, and `md5` features; the default feature set enables all three.
- Keep CLI output useful for long-running directory scans. Progress output is controlled through the `console_progress` argument in library calls.
- Do not introduce global mutable state; checksum creation is parallelized with Rayon.

## Packaging And Releases

- Keep `Cargo.toml`, `Cargo.lock`, `README.md`, `.github/workflows/build_rust.yml`, and `pkgbuild/PKGBUILD` consistent when changing release behavior or version numbers.
- If `pkgbuild/PKGBUILD` changes its source archive or version, update `.SRCINFO` as well.
- The GitHub release workflow runs on tags matching `v*.*.*` and builds `backup-checker` for Linux, Windows, and macOS.

## Verification Expectations

- For behavioral changes, add or update tests when feasible. This repository currently has little test coverage, so new tests are preferred over manual-only verification.
- At minimum, run formatting and the relevant Cargo check/test commands before handing off.
- If a command cannot be run because dependencies, network access, or platform tooling are unavailable, report that explicitly.

## Safety Notes

- This tool reads entire files to calculate some checksums. Be careful with changes that increase memory usage on large backup sets.
- Missing-file detection is checksum-based, not path-based. Duplicate files with the same checksum are intentionally treated as present if any matching content exists in the new tree.
- Be cautious with recursive traversal changes; `max_depth <= 0` must stop recursion.
