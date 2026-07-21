# Finalize the owner-voice AEP narration

## Context

Radia's automated Build Week video needs a concise AEP-first narration. The
owner selected LocalTTS review candidate 1 as exceptional and asked the demo to
show that AEP also governed the local voice workflow.

User decisions (2026-07-21): use candidate 1's underlying owner reference and
generation settings; keep AEP as the subject and Radia as proof; include
consent, dependency isolation, reference conditioning, and tests as another AEP
example; present the final script for owner review before full synthesis or OBS
recording.

## Evidence

- The current cue sheet requires full draft approval before recording -
  `docs/hackathon/demo-script.md:3-12`.
- The current narration manifest has eight sections and a fixed 165-second
  duration - `docs/hackathon/tts-narration.json:1-78`.
- The current draft is 386 words, requiring about 140 words per minute overall;
  verified from the manifest on 2026-07-21.
- The selected LocalTTS output is 18.6 seconds and records reference SHA-256
  `a582abaac44b058aefc1a7d5fea0181cf140928b22e122933a838516d880406c` -
  `C:/LocalTTS/Temp/voice-review/owner-reference-20260721-01/reference-review.provenance.json`.
- LocalTTS can review explicit Chatterbox references but full manifest
  synthesis does not yet accept one -
  `C:/LocalTTS/src/local_tts/engine.py:116-160` and
  `C:/LocalTTS/src/local_tts/cli.py:169-194`.
- Radia's README already treats full WAV listening as a separate approval gate
  before recording - `README.md:403-422`.
- The public closing card currently describes only Radia's project results -
  `docs/hackathon/video-deck.html:86-91`.

## Steps

### 1. Persist selected reference conditioning (first action)

Extend LocalTTS's existing Chatterbox adapter and `synthesize` CLI with one
optional project-local reference WAV. Reuse the selected candidate's source
reference, not the generated candidate output, to avoid conditioning a clone on
another clone. Record only the reference hash in engine provenance. Keep all
existing unconditioned manifests valid.

### 2. Rewrite and align the AEP-first script (blocked on step 1)

Replace the exact read-aloud draft and JSON section text together. Target about
40-50 words per minute across the 165-second timeline, short sentences, one
claim per cue, explicit `Codex`, `GPT-5.6`, `Developer Tools`, and a new segment
showing AEP governance of LocalTTS consent, dependencies, conditioning, and
tests. Preserve all technical claim boundaries in the detailed checklist.

### 3. Update public cards and reproduction guidance (blocked on step 2)

Add the governed voice workflow to the closing card and teleprompter cues.
Update the README synthesis command with the selected underlying reference WAV
and explain why candidate 1 itself is not the conditioning input.

### 4. Validate and stop for owner review (blocked on step 3)

Run LocalTTS tests, Radia narration/JSON agreement checks, timeline and privacy
tests, ADR/pin checks, plan checks, and ASCII checks. Report exact text, section
word counts, total words, and implied overall words per minute. Do not synthesize
the full WAV or contact OBS until the owner approves the script.

## Files to touch

- `C:/LocalTTS/src/local_tts/engine.py` (selected reference on full synthesis)
- `C:/LocalTTS/src/local_tts/cli.py` (optional `--reference-audio` input)
- `C:/LocalTTS/tests/` (reference-conditioned CLI/engine provenance cases)
- `C:/LocalTTS/README.md` (full conditioned synthesis command)
- `docs/hackathon/demo-script.md` (owner-review read-aloud draft)
- `docs/hackathon/tts-narration.json` (exact cue-aligned text and settings)
- `docs/hackathon/video-deck.html` (voice-work proof and cue copy)
- `docs/hackathon/voice-selection.json` (approved candidate contract and gates)
- `README.md` (selected reference reproduction and approval boundary)
- `CODE.md` (reviewed current AEP convention pins)
- `scripts/tests/record-devpost-demo.Tests.ps1` (script/voice alignment gate)
- `docs/plans/finalize-owner-voice-aep-narration.md` (execution contract)

## Verification

1. `pwsh -NoProfile -File C:\LocalTTS\scripts\verify.ps1` - expected: compile
   and all LocalTTS unit tests exit 0 without inference.
2. Parse `demo-script.md` and `tts-narration.json` - expected: eight read-aloud
   sections match exactly, total word count is reported, and every section fits
   its declared cue at the selected sample's measured speaking rate.
3. Run `scripts/tests/record-devpost-demo.Tests.ps1` through Pester - expected:
   all timeline, privacy, OBS-kind, and stop-recovery tests pass.
4. `agent-code-skills adr check`, `pins --check`, and both plan checks -
   expected: zero findings or drift.
5. ASCII scan across changed public text/code - expected: zero non-ASCII bytes.
6. Owner reads the exact draft in chat - expected: explicit approve or revision
   request before full synthesis. Gated: owner review.

## Non-goals

- Full narration synthesis, WAV replacement, OBS rehearsal/recording, YouTube
  upload, Devpost mutation, submission, commit, or push.
- Reconditioning on generated candidate 1 audio.

## Risks

- Voice cadence overruns a short cue -> keep each section below the selected
  sample's measured words-per-minute capacity and verify after owner approval.
- Voice-work detail displaces AEP's definition -> define AEP first; present
  LocalTTS as one concrete governed-project example near the close.
- Private source paths reach the public deck -> deck contains concepts only;
  local paths remain in owner-side README commands and gitignored provenance.
- Existing dirty video automation work is overwritten -> patch only the exact
  narration, deck, and README blocks while preserving unrelated changes.

## Open questions
