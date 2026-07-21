# Complete the Build Week presentation video

## Context

Radia has eight owner-approved synthetic narration sections, but OBS consumes
one media file. Finish the under-three-minute Build Week video by assembling
those immutable sections, retiming the visual sequence, rehearsing, recording,
and validating the result.

User decisions (2026-07-21): all eight current section WAVs are accepted;
regenerate none of them; complete the video now. Keep YouTube upload, Devpost
mutation, final submission, and final visual acceptance as owner gates.

## Evidence

- The accepted section provenance records all eight hashes, seeds, and measured
  durations - `Temp/build-week-video/tts-review/owner-reference-20260721-01/edited-take-01/sections.provenance.json:56-162`.
- The current timeline starts narration at fixed cue boundaries and stops at
  165 seconds - `docs/hackathon/video-timeline.json:10-11` and
  `docs/hackathon/video-timeline.json:104-112`.
- OBS accepts one required narration media path and restarts it with the visual
  clock - `scripts/record-devpost-demo.psm1:1133-1144` and
  `scripts/record-devpost-demo.psm1:1410-1453`.
- The accepted recording ADR keeps owner review, upload, Devpost mutation, and
  submission as manual gates -
  `docs/adr/6f6bc4ff-obs-build-week-recording-automation.md:50-51`.

## Steps

### 1. Freeze the final schedule

Place each accepted section without overlap, preserve scene order, show every
buffer mode, leave at least five uninterrupted final renderer seconds, and stop
at 160 seconds. Keep the 175-second emergency ceiling.

### 2. Assemble accepted audio

Build one mono 24 kHz PCM16 WAV from the approved files and scheduled starts.
Reject hash, format, overlap, or duration drift. Record source hashes, offsets,
output hash, output format, and duration in a local provenance JSON file.

### 3. Synchronize public presentation inputs

Update the timeline, exact read-aloud section, teleprompter prompts, and README
commands. Preserve technical checklist claims and the raw-evidence boundary.

### 4. Run mechanical gates

Validate audio provenance and the timeline, run Pester, project verification,
ADR/pin/doctor checks, plan checks, matrix ban, and an OBS-free dry-run.

### 5. Rehearse and record

Use authenticated OBS WebSocket control with the dedicated profile and scene
collection. Rehearse once without recording, then record MKV, await MP4 remux,
and generate recording provenance. Stop safely on any cue or OBS failure.

### 6. Stop for owner review

Present the MP4 and manifest. Do not upload or mutate Devpost until the owner
watches the full video and explicitly approves it.

## Files to touch

- `docs/hackathon/video-timeline.json` (final non-overlapping schedule)
- `docs/hackathon/demo-script.md` (accepted exact script and timing)
- `docs/hackathon/video-deck.html` (accepted prompt text)
- `README.md` (assembly and final recording commands)
- `Temp/build-week-video/` (assembled audio, provenance, recordings)
- `docs/plans/complete-build-week-video.md` (execution contract)

## Verification

1. Validate assembled audio - expected: eight matching source hashes, mono
   PCM16 24 kHz, no overlap, exact 160000 ms output, and matching output hash.
2. `pwsh -NoProfile -File scripts/tests/record-devpost-demo.Tests.ps1` -
   expected: all Pester cases pass.
3. `pwsh -NoProfile -File scripts/record-devpost-demo.ps1 -Action DryRun
   -NarrationPath <assembled.wav>` - expected: 160000 ms, timeline valid, and
   `ObsStarted=False`.
4. `cargo test --workspace --all-features` and the matrix-ban script -
   expected: exit 0.
5. `agent-code-skills adr check`, `pins --check`, `doctor`, and both plan checks
   - expected: exit 0 without findings.
6. Gated OBS rehearsal - expected: every scene/mode cue executes and no file is
   recorded.
7. Gated OBS recording and validation - expected: MKV, MP4, and manifest exist;
   duration is 160 seconds within controller tolerance; no material OBS lag.
8. Gated owner review - expected: intelligible audio, readable modes, no visual
   artifact or privacy leak, and explicit approval before upload.

## Non-goals

- Regenerating accepted speech, changing renderer behavior, committing,
  pushing, uploading to YouTube, mutating Devpost, or submitting.

## Risks

- Cue drift -> derive assembly offsets from the same timeline OBS executes.
- OBS/source mismatch -> dry-run first and abort recording on any missed ack.
- Visual artifact -> preserve failed MKV, diagnose, and rerecord only after fix.
- Dirty worktree loss -> patch only named files and preserve unrelated changes.
