# Releasing

AgentEye uses two GitHub Actions workflows. Both run on GitHub's free hosted runners (`ubuntu-latest`, `windows-latest`).

## Pull requests: Validate

Every pull request runs `.github/workflows/ci.yml`:

| Job | What it checks |
|---|---|
| UI build and test | `npm run build` and `npm test` in `ui` (Windows) |
| Rust (ubuntu-latest, windows-latest) | `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` |

Make these required checks in the repository's branch protection rules for `main`.

## Merges to main: release candidates

Each commit that lands on `main` is validated again. If everything passes, the same workflow:

1. Works out the version this commit would release (see Versioning below).
2. Builds installers on Windows (NSIS `.exe`) and Linux (`.deb` and `.AppImage`) with that version stamped in.
3. Publishes a **pre-release** tagged `X.Y.Z-rc.N` with the installers and a `.sha256` file for each.
4. Deletes older candidates, keeping the **newest 3**.

Candidates are for trying a build, not for end users.

## Manual release: Release

When a candidate is good, run **Actions → Release → Run workflow** on `main`.

It promotes the candidate for the latest `main` commit. It does **not** rebuild: it downloads that candidate's installers,
verifies their checksums, and republishes the identical files as release `X.Y.Z` (marked latest, with generated notes since the
previous release). The candidate suffix is dropped from the file names only. The candidates of that version are then removed.

If `main` has moved since the candidate you wanted, the workflow promotes the newest commit's candidate; wait for Validate to
finish on it first.

## Versioning

- `version.json` holds `major.minor` (for example `0.1`). Edit it by hand to start a new minor or major line.
- The patch is computed: one more than the newest release tag of that line, or `0` for a new line. Releasing `0.1.0` makes the
  next candidates `0.1.1-rc.1`, `0.1.1-rc.2`, and so on.
- The version is **stamped into the build, not committed**: `eng/Set-Version.ps1` writes it to the Cargo workspace and
  `src-tauri/tauri.conf.json` and both `package.json` files on the CI checkout. The installers therefore carry the release version (`0.1.1`), never an
  `-rc` suffix, so promoting a candidate does not change them. The `-rc.N` appears only in the tag and file names.
- The version in the committed files is a development placeholder.

## Running the scripts locally

PowerShell 7 is required (`winget install Microsoft.PowerShell`).

```powershell
./eng/Get-NextVersion.ps1             # next release version (candidate number needs -Candidate and gh)
./eng/Set-Version.ps1 -Version 0.1.0  # rewrites Cargo.toml, tauri.conf.json and the package.json files; revert with git checkout
```

## One-time repository setup

- Settings → Actions → General: allow actions, and leave workflow permissions at the default (each job declares what it needs).
- Settings → Branches: protect `main`, require pull requests and the checks above.
- No secrets are needed; releases use the built-in `GITHUB_TOKEN`.
- Installers are not code-signed yet, so Windows SmartScreen will warn on first run.
