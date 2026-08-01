# Versioning

This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html) with an explicit **release channel** suffix, and [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## Surface

optionSDK is a single surface: the shared Rust crate published to [crates.io](https://crates.io/crates/optionSDK). Changelog headings use the channel suffix, e.g. `## [0.1.1-stable] - YYYY-MM-DD`.

`Cargo.toml` / git tags / crates.io keep the numeric version (`0.1.1`, `v0.1.1`) — the channel lives in the changelog (and release notes), matching optionMusic’s channel-labeled releases.

## Release channels (`x.y.z-<channel>`)

| Channel | Tag example | Meaning |
|---------|-------------|---------|
| **alpha** | `0.2.0-alpha` | Extremely early. Features incomplete; bugs are expected and common. |
| **beta** | `0.2.0-beta` | Feature set nearly complete, but still rough — bugs and hard edges remain. |
| **stable** | `0.1.1-stable` | Production-ready: finished for that version, few or no known bugs. |

Do **not** label something `stable` unless it is actually release-ready. Prefer **beta** while a large API rewrite is settling; use **alpha** only for brand-new / half-built surfaces.

Alpha/beta cuts are normally changelog + local/dev artifacts — not GitHub Release / crates.io — unless explicitly promoted to **stable**.
