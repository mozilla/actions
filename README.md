# actions

GitHub actions shared by various Mozilla projects.

## Composite Actions

### `rust` — Install Rust and tools

Installs a Rust toolchain with optional components and tools. Uses
[`Swatinem/rust-cache`](https://github.com/Swatinem/rust-cache) to cache
dependencies (one entry per OS × toolchain, saves only on the default branch). Handles
MSVC setup on Windows.

```yaml
- uses: mozilla/actions/rust@v1
  with:
    version: stable # Toolchain version (default: stable)
    components: clippy # Space-separated Rust components
    tools: cargo-nextest # Comma- or space-separated tools (installed via cargo-binstall)
    token: ${{ github.token }} # GitHub token to avoid API rate limits
    targets: aarch64-unknown-linux-gnu # Comma-separated target triples
    rust-cache: true # Whether to enable rust-cache (default: true; auto-disabled when sccache: true)
    sccache: false # Whether to enable sccache (default: false)
```

### `toolchains` — Determine Rust toolchains from MSRV

Reads `rust-version` from `Cargo.toml` and outputs a JSON array
`["<msrv>", "stable", "nightly"]` for use in CI matrices.

```yaml
- uses: mozilla/actions/toolchains@v1
  id: toolchains
  with:
    working-directory: . # Directory containing Cargo.toml (default: .)

# Use in matrix:
# strategy:
#   matrix:
#     toolchain: ${{ fromJSON(steps.toolchains.outputs.toolchains) }}
```

### `nss` — Install Mozilla NSS

Installs Mozilla's Network Security Services (NSS) library. Uses the system
package if it meets the minimum version requirement; otherwise downloads and
builds from source with caching.

Sets environment variables: `NSS_DIR`, `NSS_PREBUILT`, `LD_LIBRARY_PATH`
(Linux), `DYLD_FALLBACK_LIBRARY_PATH` (macOS).

```yaml
- uses: mozilla/actions/nss@v1
  with:
    minimum-version: "3.100" # Minimum required NSS version
    # OR
    version-file: nss/min_version.txt # File containing the minimum version
    target: "" # Cross-compilation target (e.g. aarch64-linux-android)
    sccache: false # Whether to enable sccache for NSS compilation (default: false)
```

If the `rust` action was called with `sccache: true` earlier in the same job, the `nss`
action will detect this automatically and use sccache for the NSS build without needing
`sccache: true` here.

## Reusable Workflows

Call these from a job in your workflow using `uses:`. Workflows that depend on
NSS require callers to run `mozilla/actions/nss@v1` in a prior step.

```yaml
jobs:
  deny:
    uses: mozilla/actions/.github/workflows/deny.yml@v1
  rustfmt:
    uses: mozilla/actions/.github/workflows/rustfmt.yml@v1
  machete:
    uses: mozilla/actions/.github/workflows/machete.yml@v1
  actionlint:
    uses: mozilla/actions/.github/workflows/actionlint.yml@v1
  dependency-review:
    uses: mozilla/actions/.github/workflows/dependency-review.yml@v1
  clippy:
    uses: mozilla/actions/.github/workflows/clippy.yml@v1
    with:
      exclude-features: gecko # optional
  sanitize:
    uses: mozilla/actions/.github/workflows/sanitize.yml@v1
    with:
      features: ci # optional
  mutants-pr:
    uses: mozilla/actions/.github/workflows/mutants-pr.yml@v1
  mutants:
    uses: mozilla/actions/.github/workflows/mutants.yml@v1
  semver:
    uses: mozilla/actions/.github/workflows/semver.yml@v1
    with:
      package: my-crate # optional; omit to check all packages
```

### `deny.yml` — cargo deny

Runs [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) to check for
security advisories, banned crates, license compliance, and allowed sources.
Advisory checks use `continue-on-error` to avoid blocking CI on sudden
announcements. Requires a
[`deny.toml`](https://embarkstudios.github.io/cargo-deny/checks/index.html)
in the repository root.

### `rustfmt.yml` — Formatting

Runs `cargo fmt --all -- --check` with nightly rustfmt.

### `machete.yml` — Unused dependencies

Runs [`cargo-machete`](https://github.com/bnjbvr/cargo-machete) and
`cargo-hack` to find unused dependencies across all workspace crates and
feature combinations.

### `actionlint.yml` — Lint GitHub Actions workflows

Runs [`actionlint`](https://github.com/rhysd/actionlint) and
[`zizmor`](https://github.com/woodruffw/zizmor) on changes to workflow and
composite action files. Triggers automatically on pull requests.

### `dependency-review.yml` — Dependency review

Runs the [GitHub Dependency Review Action](https://github.com/actions/dependency-review-action)
to surface known-vulnerable package versions introduced in a PR.

### `clippy.yml` — Clippy

Runs `cargo hack clippy --feature-powerset` across a matrix of OS (Linux,
macOS, Windows) and toolchains (MSRV, stable, nightly), plus `cargo doc` with
strict warnings. Accepts an `exclude-features` input for project-specific
features to exclude from the powerset (e.g. `gecko`).

### `sanitize.yml` — Sanitizers

Runs tests with address, thread, and leak sanitizers on Linux and macOS using
nightly Rust. Accepts a `features` input to enable project-specific Cargo
features during testing. macOS leak sanitizer suppresses known system library
leaks automatically.

### `mutants-pr.yml` — PR mutation testing

Runs [`cargo-mutants`](https://mutants.rs) on the diff introduced by a PR,
checking that each mutation is caught by the test suite. Posts results as a
job summary.

### `mutants.yml` — Full mutation testing (scheduled)

Runs `cargo-mutants` across the entire codebase in parallel shards
(configurable via `shards` input). Designed for weekly scheduled runs. Merges
shard results and posts a summary with missed/caught/timeout counts.

### `semver.yml` — Semver compatibility

Runs [`cargo-semver-checks`](https://github.com/obi1kenobi/cargo-semver-checks)
against the PR base branch to catch breaking API changes. Accepts a `package`
input to check a specific crate (omit to check all packages).

## Versioning

Actions and workflows are versioned with `@v1` tags. Pin to a tag for stability:

```yaml
- uses: mozilla/actions/rust@v1
```

or to a specific commit SHA for reproducibility:

```yaml
- uses: mozilla/actions/rust@<sha>
```
