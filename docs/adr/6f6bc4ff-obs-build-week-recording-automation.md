---
id: 6f6bc4ff-ec39-4116-a83d-e539ff4ab367
slug: adr:obs-build-week-recording-automation
title: OBS Build Week recording automation
status: accepted
supersedes: []
supersededBy: null
deciders: ["LairdWT"]
proposedAt: 2026-07-20
decidedAt: 2026-07-20
tags: []
---

## Context

The public Radia renderer is the Build Week reference project for Agent
Enhanced Projects, but the required narrated video is not yet recorded. The
existing OBS collection captures the whole desktop, the renderer exposes only
interactive keyboard controls, and a manually timed take risks private-window
exposure, unreadable proof, mode drift, and exceeding the three-minute limit.
The video presents existing controlled GPU evidence; it must not be treated as
a replacement for the authoritative raw captures and manifests.

## Decision

Automate the owner-side Build Week recording with OBS Studio 30.2.3 and its
built-in WebSocket 5 protocol, controlled by a Windows-only PowerShell 7.4+
script. Keep WebSocket authentication required, accept its credential only as
an in-memory `PSCredential`, and never place the password in source, command
arguments, output, or logs.

Add a dependency-free `radia-demo present --control-stdin` mode. It owns one
bounded standard-library command channel and accepts explicit ASCII `mode`,
`reset`, and `quit` commands with flushed machine-readable acknowledgements.
Keep the normal interactive window behavior unchanged. Presentation mode uses
a stable title and continues to derive dragon and light motion analytically
from one monotonic elapsed-time clock.

Use a dedicated `Radia Build Week` OBS profile and scene collection at
1920x1080, 30 fps, Rec.709 SDR, NVENC H.264, and AAC stereo. Record to MKV and
automatically remux to MP4. Capture only the Radia window and project-owned
local browser cards; never use display capture, live editor or terminal
capture, or a facecam. Record narration first, then replay it as the only final
audio source while a versioned timeline drives scenes and explicit Radia
modes. Preserve the user's prior OBS profile and collection and refuse unknown
reserved-name collisions.

Treat the resulting video and its hash manifest as presentation provenance,
not deterministic visual acceptance. The committed direct GPU captures and
AEP-compatible manifests remain numeric authority. Owner review, YouTube
upload, Devpost mutation, and final submission remain explicit manual gates.

## Consequences

The final take is repeatable, privacy-bounded, synchronized to the cue sheet,
and hard-stopped below three minutes without global keystrokes or mouse
automation. The repository gains a small presentation control interface, a
public proof deck, a versioned timeline, PowerShell automation, tests, and
README recovery instructions. Recording requires Windows, PowerShell 7.4+,
OBS 30.2.3-compatible WebSocket 5, authenticated local access, an available
NVENC H.264 encoder, and one owner narration take. Compressed video bytes are
not expected to repeat across takes or replace fixed-state renderer evidence.

## Alternatives

Full-desktop capture was rejected because it can expose unrelated private
state. Global `SendKeys` or mouse automation was rejected because focus and
timing are nondeterministic. Live editor and terminal footage was rejected
because dense text is unreadable at judge playback speed and increases privacy
risk. Live narration during the final visual take was rejected because wording
liberties would desynchronize scene timing. Direct MP4 recording was rejected
because an interrupted file is less recoverable than MKV. A new OBS client
module or Rust crate was rejected because the standard .NET WebSocket and Rust
standard library provide the required bounded interfaces.
