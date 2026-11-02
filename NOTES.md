# Session notes

Read this first in every session; update it at the end. The full design record (decisions
D1–D16, risks, phases) lives in Jorge's "Unified editor — architecture decision record" doc.

## Rules

- **Branches:** one branch per session (`phase-N/<topic>`), merged into `main` only when the
  Windows build and tests pass. Jorge downloads builds from `main`.
- **Pull before work, push when done**, on both sides.
- Build and test on Windows through GitHub Actions; the cloud sandbox only checks small things.
- A session ends with the branch building and committed.

## Where we are (2 Nov 2026)

**Phase 0 closed.** Baseline on Jorge's laptop (6D Mark II CR2s, medians): LightCraft 2048 px
render 0.95 s CPU / 2.1 s GPU (GPU number includes device start-up per process); full-size
render 4.0 s CPU / 5.1 s GPU at about 1.9 GB peak; PhotoCraft opens a CR2 in 3.8 s / 0.9 GB.

**Phase 1 (one workspace, cleanup), in progress** on `phase-1/one-workspace`:

- [x] One Cargo workspace at the root; dependency versions and features unchanged per app
      (checked with `cargo tree`), except four patch-level bumps listed in the commit.
- [x] Windows CI builds and tests each app from the shared workspace.
- [x] Removed non-Windows packaging (macOS, Linux, FreeBSD), Nix and Docker files, and the
      upstream workflows in each app folder (GitHub only runs the root ones).
- [ ] Remaining D8 dead code, deferred to the cleanup Jorge postponed: macOS/Linux `cfg` code
      paths in Rust, `lightcraft-sysmem` (a macOS memory call), the cut candidates.
- [ ] Fix LightCraft's 21 Windows test failures (file moves, renames, XMP sidecar repair).
      Until then: don't move or rename photos with LightCraft.
- [ ] Fix PhotoCraft's 4 Windows test failures.
- [ ] LightCraft can't read a 144 MB 16-bit TIFF ("decoder limits exceeded"); PhotoCraft can.
- Gate: one Windows CI run green; renders unchanged.

**Waiting on Jorge:** confirm the cut candidates (both web apps, Affinity import; geocode and
WASM plug-ins stay unless he says otherwise).

## Logged for later phases

- Phase 2 (raw layer): standalone `.dcp` loading (his X-Rite profiles are matrix-only, one per
  session, in `%APPDATA%\Adobe\CameraRaw\CameraProfiles`) with batch apply; lens profile looked
  up at render time or re-detectable, with a manual camera/lens choice and the Lensfun data
  version recorded per edit; auto-crop after lens correction for each crop variant; dehaze is a
  port of PhotoCraft's.
- Merged UI: a visible Selection Brush (Quick Mask painting); edits always pre-empt background
  preview work, with a busy indicator.
- After the MVP: large-object removal (tune PatchMatch first; LaMa only if Jorge decides AI may
  fill pixels); lens calibration module writing Lensfun XML.

## Test files

- `C:\UE\photos` on Jorge's laptop: baseline set (5 CR2, 1 PSD, 1 TIFF).
- `C:\UE\Profiles`: colour acceptance test: `2026-09-19 12-04-58 - 1675` CR2, its DNG and
  Lightroom JPG (profile 19-09-2026, 5200 K / +17, EF 50mm f/1.8 STM profile).
- Test files never go into this public repo.
