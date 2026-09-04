# actions

GitHub actions shared by various Mozilla projects.

## Composite Actions

### `rust` — Install Rust and tools

Installs a Rust toolchain with optional components and tools. Uses
[`Swatinem/rust-cache`](https://github.com/Swatinem/rust-cache) to cache
dependencies (one entry per OS × toolchain, saves only on the default branch, and not for
rolling `nightly`). PR and merge-queue runs restore but never save, so a workflow that never
runs on the default branch gets no caching at all. Handles MSVC setup on Windows.

```yaml
- uses: mozilla/actions/rust
  with:
    version: stable # Toolchain version (default: stable)
    components: clippy # Space-separated Rust components
    tools: cargo-nextest # Comma- or space-separated tools (installed via cargo-binstall)
    token: ${{ github.token }} # GitHub token to avoid API rate limits
    targets: aarch64-unknown-linux-gnu # Comma-separated target triples
    rust-cache: true # Whether to enable rust-cache (default: true; auto-disabled when sccache: true, and for rolling nightly)
    cache-key: "" # Extra cache key component, to split jobs that would otherwise share an entry
    sccache: false # Whether to enable sccache (default: false)
    make-default: true # Whether to make this toolchain the default (default: true)
```

### `toolchains` — Determine Rust toolchains from MSRV

Reads `rust-version` from `Cargo.toml` and outputs a JSON array
`["<msrv>", "stable", "nightly"]` for use in CI matrices.

```yaml
- uses: mozilla/actions/toolchains
  id: toolchains
  with:
    working-directory: . # Directory containing Cargo.toml (default: .)

# Use in matrix:
# strategy:
#   matrix:
#     toolchain: ${{ fromJSON(steps.toolchains.outputs.toolchains) }}
```

### `claude-review` — Claude Code Review

Runs [Claude Code](https://claude.ai/code) to perform an AI-assisted code review on a pull
request. Posts inline comments and a PR-level summary via the GitHub review API. A pull
request from a fork is reviewed only if its author has write access, because reviewing one
means checking out its code; other pull requests are skipped and the workflow still
succeeds, without posting a review.

Claude takes `.claude` (settings, skills, agents, commands) and
`.github/copilot-instructions.md` from the base branch rather than from the pull request,
and ignores `CLAUDE.md`, `CLAUDE.local.md`, `AGENTS.md` and `.mcp.json` entirely. Use
`.github/copilot-instructions.md` on the base branch, or the `prompt` input, for
project-specific instructions.

> [!NOTE]
> Requires an `ANTHROPIC_API_KEY` repository secret.

The easiest way to use this is to copy [`.github/workflows/claude-review.yml`](.github/workflows/claude-review.yml)
into your repository — it includes the trigger and permission gating. Add a
[concurrency group](https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/control-the-concurrency-of-workflows-and-jobs)
and an `ANTHROPIC_API_KEY` secret and you're done.

Alternatively, call it as a [reusable workflow](https://docs.github.com/en/actions/sharing-automations/reusing-workflows)
using `secrets: inherit`. Or use the composite action directly to customize model, budget, or prompt:

```yaml
- uses: mozilla/actions/claude-review
  with:
    anthropic_api_key: ${{ secrets.ANTHROPIC_API_KEY }} # zizmor: ignore[secrets-outside-env]
    prompt: "Focus on protocol compliance and unsafe FFI usage." # optional
```

| Input               | Default      | Description                                        |
| ------------------- | ------------ | -------------------------------------------------- |
| `anthropic_api_key` | _(required)_ | Anthropic API key                                  |
| `model`             | `""`         | Primary Claude model (upstream default when unset) |
| `fallback_model`    | `""`         | Fallback model (upstream default when unset)       |
| `budget`            | `5.00`       | Max spend per review in USD                        |
| `prompt`            | `""`         | Additional project-specific review instructions    |

### `crap` — CRAP analysis

Runs [`cargo-crap`](https://github.com/minikin/cargo-crap) to compute
[CRAP (Change Risk Anti-Patterns)](https://testing.googleblog.com/2011/02/this-code-is-crap.html)
scores for Rust functions. CRAP combines cyclomatic complexity with test
coverage — high-complexity, low-coverage functions score above the threshold.
Installs nightly Rust, `cargo-llvm-cov`, and `cargo-crap` automatically.

```yaml
- uses: mozilla/actions/crap
  with:
    features: ci # Cargo features for coverage (optional)
    threshold: "30" # CRAP score threshold (default: 30)
    output: crap.sarif # SARIF output path (default: crap.sarif)
    token: ${{ github.token }} # Avoid API rate limits (optional)
    lcov-path: lcov.info # Use pre-existing LCOV instead of generating (optional)
```

### `semver` — Semver compatibility

Runs [`cargo-semver-checks`](https://github.com/obi1kenobi/cargo-semver-checks)
against a baseline revision to catch breaking API changes.

```yaml
- uses: mozilla/actions/semver
  with:
    package: my-crate # optional; omit to check all packages
    base-ref: origin/main # optional; defaults to the PR base or default branch
```

### `nss` — Install Mozilla NSS

Installs Mozilla's Network Security Services (NSS) library. Uses the system
package if it meets the minimum version requirement; otherwise downloads and
builds from source with caching.

Sets environment variables (when not using system NSS): `NSS_DIR`, `NSS_PREBUILT`,
`NSS_BUILD_FLAGS`, `NSS_BUILD_CFLAGS`, `LD_LIBRARY_PATH` (Linux),
`DYLD_FALLBACK_LIBRARY_PATH` (macOS).

```yaml
- uses: mozilla/actions/nss
  with:
    minimum-version: "3.100" # Minimum required NSS version
    sha256: "..." # SHA256 of the 'nss-3.100-with-nspr-<version>.tar.gz' release tarball
    target: "" # Cross-compilation target (e.g. aarch64-linux-android)
    deps-only: false # Install the build toolchain only, acquiring no NSS (default: false)
    sccache: false # Whether to enable sccache for NSS compilation (default: false)
    token: ${{ github.token }} # GitHub token to avoid API rate limits (needed for Android builds)
```

`deps-only: true` installs the per-platform NSS build toolchain (gyp everywhere, ninja
on Linux and macOS, and nsinstall plus the MSVC and msys2 setup on Windows, which is
expected to supply ninja itself) and sets the environment variables above, but
retrieves and builds nothing. A caller that puts its own NSS checkout at `$NSS_DIR`
can then build it the way this action does:

```bash
[ "$SCCACHE_CC" ] && [ "$SCCACHE_CXX" ] && export CC="$SCCACHE_CC" CXX="$SCCACHE_CXX"
[ -n "$NSS_BUILD_CFLAGS" ] && export CFLAGS="$NSS_BUILD_CFLAGS"
"$NSS_DIR/build.sh" $NSS_BUILD_FLAGS
```

`minimum-version`, `sha256` and `cache` are unused in that mode, and
`NSS_PREBUILT` and the library paths point at a `dist/` the caller has yet to
produce. `target: *-android` is rejected, because Android builds do not go through
`build.sh`.

If the `rust` action was called with `sccache: true` earlier in the same job, the `nss`
action will detect this automatically and use sccache for the NSS build without needing
`sccache: true` here.

## Reusable Workflows

Call these from a job in your workflow using `uses:`. Workflows that depend on
NSS require callers to run `mozilla/actions/nss` in a prior step.

```yaml
jobs:
  claude-review:
    uses: mozilla/actions/.github/workflows/claude-review.yml
    permissions:
      contents: read
      pull-requests: write
      issues: read
      actions: read
      discussions: read
    secrets: inherit
  deny:
    uses: mozilla/actions/.github/workflows/deny.yml
  rustfmt:
    uses: mozilla/actions/.github/workflows/rustfmt.yml
  machete:
    uses: mozilla/actions/.github/workflows/machete.yml
  actionlint:
    uses: mozilla/actions/.github/workflows/actionlint.yml
    permissions:
      contents: read
      security-events: write # Required for zizmor to upload SARIF results
  dependency-review:
    if: github.event_name == 'pull_request'
    uses: mozilla/actions/.github/workflows/dependency-review.yml
  clippy:
    uses: mozilla/actions/.github/workflows/clippy.yml
    with:
      exclude-features: gecko # optional
  sanitize:
    uses: mozilla/actions/.github/workflows/sanitize.yml
    with:
      features: ci # optional
  crap:
    uses: mozilla/actions/.github/workflows/crap.yml
    permissions:
      contents: read
      security-events: write # Required to upload SARIF results to GitHub
    with:
      features: ci # optional
      threshold: 30 # optional
  mutants-pr:
    uses: mozilla/actions/.github/workflows/mutants-pr.yml
  mutants:
    uses: mozilla/actions/.github/workflows/mutants.yml
  sbom:
    uses: mozilla/actions/.github/workflows/sbom.yml
    permissions:
      actions: read # Required by anchore/sbom-action to read workflow run context.
      contents: write # Required to upload the SBOM as a release asset.
  release:
    uses: mozilla/actions/.github/workflows/release.yml
    permissions:
      contents: write # Required to update the major version tag
    with:
      tag: ${{ github.event.release.tag_name }}
```

### `claude-review.yml` — Claude Code Review

Wraps the [`claude-review`](#claude-review--claude-code-review) composite action as a
self-contained workflow. Handles the `pull_request_target` trigger and permission gating
(`OWNER`/`MEMBER`/`COLLABORATOR` only). Concurrency is the caller's responsibility. Can be
copied directly into a repository or called as a reusable workflow with `secrets: inherit`.
To customize model, budget, or prompt, use the composite action directly.

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

### `crap.yml` — CRAP analysis

Runs [`cargo-crap`](https://github.com/minikin/cargo-crap) across a matrix of
OS (Linux, macOS, Windows) to compute CRAP scores. Generates test coverage via
`cargo-llvm-cov`, then uploads results as SARIF to GitHub Code Scanning. On
pull requests, GitHub automatically highlights new high-CRAP functions
(regressions from the default branch baseline). Accepts a `features` input to
enable project-specific Cargo features during coverage, and a `threshold` input
to control the CRAP score cutoff (default: 30).

### `mutants-pr.yml` — PR mutation testing

Runs [`cargo-mutants`](https://mutants.rs) on the diff introduced by a PR,
checking that each mutation is caught by the test suite. Posts results as a
job summary.

### `mutants.yml` — Full mutation testing

Runs `cargo-mutants` across the entire codebase in parallel shards
(configurable via `shards` input). Designed for scheduled runs — callers must
provide their own `schedule` trigger. Merges shard results and posts a summary
with missed/caught/timeout counts.

### `sbom.yml` — SBOM generation

Wraps Mozilla's [`ssdlc-sbom`](https://github.com/mozilla/ssdlc-actions)
reusable workflow to generate a Software Bill of Materials for SSDLC
compliance. Centralizes the pinned `ssdlc-actions` SHA and names the SBOM after
the release tag (on release events) or commit SHA. Callers supply the trigger
(typically `release: published` and/or pushes to a release branch):

```yaml
name: SBOM
on:
  release:
    types: [published]
  push:
    branches:
      - "release/**" # adjust to your release branch pattern
  workflow_dispatch:

jobs:
  sbom:
    uses: mozilla/actions/.github/workflows/sbom.yml
    permissions:
      actions: read # Required by anchore/sbom-action to read workflow run context.
      contents: write # Required to upload the SBOM as a release asset.
```

### `release.yml` — Update major version tag

Force-moves a floating major-version tag (e.g. `v1`) to point at the same commit as an
immutable release tag (e.g. `v1.2.3`), creating the major tag if it doesn't exist yet.
Intended for repos that publish their own actions/workflows and want consumers to be
able to pin to a moving major-version alias, the same way this repo's own major-version tag
works. Callers typically trigger it from their own `release: published` event:

```yaml
name: Release
on:
  release:
    types: [published]

permissions: {}

jobs:
  release:
    uses: mozilla/actions/.github/workflows/release.yml
    permissions:
      contents: write # Required to update the major version tag
    with:
      tag: ${{ github.event.release.tag_name }}
```
