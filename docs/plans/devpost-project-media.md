# Produce the Devpost project-media pack

## Context

Create a concise set of 16:9 PNGs for manual upload to the Devpost Project
Media section. User decision (2026-07-21): cover Radia beauty renders, AEP's
project layout and workflow, and the Git repositories. The media is promotional
presentation, not authoritative GPU evidence.

## Evidence

- Radia is the public AEP reference renderer (`README.md:3-18`).
- The current renderer exposes deterministic headless capture with explicit
  dimensions, scene time, angle, and mode (`README.md:150-169`).
- Radia's current public claims include the 871,306-triangle dragon, 32-tap
  current-frame indirect gather, matrix ban, and direct GPU evidence
  (`README.md:19-56`).
- The existing proof deck establishes the navy/cyan AEP presentation language
  (`docs/hackathon/video-deck.html:6-73`).
- AEP's public-facing README defines its quickstart, Rust CLI, router skills,
  ADR lifecycle, and 21-plugin/433-skill catalog (`C:/aep/README.md:22-120`,
  `C:/aep/README.md:245-304`).
- Radia documents that the AEP library is private (`README.md:320-324`). A
  public unauthenticated check of the configured AEP origin returned 404;
  repository media must not claim that AEP is public.

## Steps

1. Capture three 1280x720 Radia frames and four buffer modes through Radia's
   project-owned headless capture adapter. Preserve the raw PNGs and hashes.
2. Build one self-contained HTML media deck using only local assets and verified
   facts. Slides: renderer hero, three-angle showcase, AEP workflow, project
   layout, repository relationship, and inspectable proof.
3. Render every slide at 1280x720 through an offline System.Drawing compositor
   into `Temp/build-week-video/project-media/`. The in-app browser rejected
   local-file rendering under its URL safety policy, so no browser workaround
   is used.
4. Write a manifest with source paths, hashes, derived-output hashes, purpose,
   fallback reason, and manual upload order; create one contact sheet.
5. Inspect all outputs. Refuse any slide with clipped text, invented UI,
   private paths, or a false public-repository claim.

## Files to touch

- `docs/plans/devpost-project-media.md` (execution contract).
- `Temp/build-week-video/project-media/project-media.html` (local media deck).
- `Temp/build-week-video/project-media/render-project-media.ps1` (offline
  compositor; no repository dependency).
- `Temp/build-week-video/project-media/manifest.json` (provenance and order).
- `Temp/build-week-video/project-media/*.png` (manual-upload media).

## Verification

1. Headless capture commands - expected: seven 1280x720 PNGs, exit 0.
2. Offline compositor - expected: six 1280x720 PNGs, exit 0.
3. Dimension/hash check - expected: every output is 1280x720, non-empty, and
   has a SHA-256 entry in `manifest.json`.
4. Privacy scan - expected: no `C:\\Users`, `C:\\Legaia`, credential, token,
   or private service data in HTML, manifest, or rendered OCR source text.
5. Human/model contact-sheet inspection - expected: all text readable; Radia
   frames show the intended triad; AEP is labeled private and Radia public.

## Non-goals

- No Devpost photo upload; the plugin does not support Project Media photos.
- No mutation of repository visibility, commit, push, or final submission.
- No replacement of raw GPU evidence with designed media.

## Risks

- Upscaling low-resolution evidence can imply false authority -> capture fresh
  1280x720 presentation frames and retain their raw hashes.
- AEP public URL currently returns 404 -> label the AEP core private and require
  judge access before final submission.

## Open questions
