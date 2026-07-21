# Owner-review Devpost submission draft

This is a local drafting aid for existing project `1345054`, **Agent Enhanced
Projects (AEPs)**. The owner authorized the ordered publication workflow.
Devpost version 8 now contains the public quaternion-deferred triad, current
controlled-delta measurements, dual-adapter Vulkan results, and fresh-clone
judge proof for Radia commit `c063b879418cf85092ace15a091a0d2e016206ee`.
The project page reports `published`. The public YouTube video and approved AEP
thumbnail are attached. OpenAI Build Week submission `1096888` was accepted as
`Submitted` on 2026-07-21 at 19:34:11 UTC.

The owner supplied the current live description in their own voice on
2026-07-21. Preserve this copy as the submission authority unless the owner
edits it again.

## Owner-authored current description

## Inspiration
I was inspired by frustration more than anything. Coding agents can be
exceptional early on in project work, however they can also tend to "do whatever
they want" when left to work unchecked, even with well made AGENTS.md files or
trying to keep "memories" for a project. So I was inspired again by the auto
mode categorizer when I had been customizing it for a project - I wanted to
make decisions, instructions, and requirements "Mechanically" enforced and not
just "Rationally" enforced. Time for some code.

## What it does
AEP serves some core roles that are split between the "Rational" agent based
features and the "Mechanical" code/script based features.

AEP comes with a framework of context light skills for many coding languages,
planning patterns, and best practices. These are then backed up by CLI tooling,
git hooks, agent definitions (Designed via the ChatGPT documentation), ADR
lifecycle management, and strict project definitions and rules that are
generated and has verified to prevent AGENTS.md and like files from being
subject to drift and tampering. They are core and spokes based designed, so the
specifics of a given skill are only fetched on demand, however the Agents are
aware of all skills via their plugin/skill *-core installations.

ADRs are a formal way to record your decisions into amend only laws that govern
your project. They are both enforced via the agents AND the mechanical tooling.
There is a strict mechanism for minting, amending, updating, superceeding, and
maintaining ADRs in the project ecosystem. This check backed decision record
become invaluable at preventing drift and banned/prevented actions. It is less
a harness, and more like rails for the train of production to run on.

State management of Temp work vs ratified Documentation and Deliverables is
essential, and AEP comes with strictly defined schemas and artifact templates
for dealing with the mundane yet very essential act of directory heigene. The
dedicated Janitor agent definition is an unassuming, but quite essential part
of the puzzle as well.

## How we built it
Painfully over a large amount of time and teration in a much larger full system
ecosystem the first iterations of what would eventually become AEP was made.
Over time, I would distill skills from my own work, or online (open source, free
information, etc) to create a strictly curated set of skills and workflows.

Once that was completed, I used CODEX and GPT 5.6 Sol to do basically all of the
heavy lifing of taking the spirit of that system and turning it inot a flexible
template to build any other type of code based project around. Then, it was
tried and tested on RAIDA and the Text to Speech demonstration shown in the
attached video.

## Challenges we ran into
A core challenge was tuning the strictness. Too many hooks, requirements, and
guard rails could spiral an agent into an infinite review loop of never
progressing. So taking the time to iterate on the tightest governance I could
build that doesnt choke out results was the largest and most time consuming
part.

Testing was difficult, as separating the "GPT is just good" vs AEP is helping
was something initially thought to be quite challenging. However, to reproduce
how much more effective AEP is, I would simply encourage a user to try to build
a full deferred rendering engine in a single sitting with just a raw agent,
then do it with AEP.  The difference after even the first hour becomes
apparent.

## Accomplishments that we're proud of
Making RADIA in a single session was amazing. Then, when runnig into micorphone
issues, getting an AEP project set up to control OBS and generate a Text to
Speech presentatino using my own voice from another one of my vidoes in a single
session was almost as impressive as the rendering engine creation. GPT 5.6 Sol
works very well within the AEP ecosystem, and felt like a design partner more
than an agent I had to order around and constantly remind about every little
detail.

## What we learned
A combination of "Rational" and "Mechanical" is the best way to set up Agent
based environments. And also having a good "Janitor" for all the artifacts and
experiments is very, very important for hard drive space.

## What's next for Agent Enhanced Projects (AEPs)
A complete Rust based game engine development system with complete MCP, LSP,
CLI, and Agent support. Singularity engine built up on the Radia rendering core
will be a fun next step!

## Live required fields

- Category: `Developer Tools`.
- Repository: `https://github.com/LairdWT/Radia` as the new reference project;
  retain `https://github.com/LairdWT/agent-enhanced-project` as the submitted
  developer tool's primary repository. AEP is not publicly accessible; grant
  the required judge accounts access or publish it before submission.
- Judge instructions: clone Radia, install Rust 1.95 and a Vulkan driver, run
  `cargo test --workspace --all-features`, then
  `cargo run --release -p radia-demo`.
- `/feedback` session ID: `019f75b8-db9b-77b3-87b3-d4870eb66651`.
- Submitter Type: `Individual` (owner supplied).
- Country of Residence: `United Kingdom` (owner supplied).
- Public YouTube video: `https://youtu.be/A5QJKrxsUS4`.
- Thumbnail: `Temp/build-week-video/youtube-thumbnail.png`; Devpost upload
  returned HTTP 200 on 2026-07-21.

## Proposed `built_with` additions

Review against the current list before replacing it:

- Codex
- GPT-5.6
- Rust
- WGPU
- WGSL
- Vulkan
- GitHub Actions

## Staged custom submission answers

- `27945` Submitter Type: `Individual`.
- `27946` Country of Residence: `United Kingdom`.
- `27947` Category: `Developer Tools`.
- `27948` Code repository:
  `https://github.com/LairdWT/agent-enhanced-project`.
- `27949` Judge link and test notes: AEP is the private primary repository above.
  Its README contains a no-build Codex plugin test plus the full mechanical
  install and verification path. Radia is the public reproducible Build Week
  reference project at `https://github.com/LairdWT/Radia`; its README provides
  the Vulkan demo and verification commands. No app credentials or sample data
  are required.
- `27950` Feedback Session ID:
  `019f75b8-db9b-77b3-87b3-d4870eb66651`.
- `27951` Developer-tool instructions: supports Windows, macOS, and Linux with
  Codex CLI 0.121+ and/or Claude Code. Clone AEP, register the local marketplace,
  install `aep@agent-code-skills` plus required domain plugins, then run
  `scripts\install.ps1` on Windows or `scripts/install.sh` on macOS/Linux to
  build the optional Rust governance CLI. Verify with `scripts\verify.cmd` or
  `sh scripts/verify.sh`. Radia can be tested separately with Rust 1.95, a
  Vulkan driver, `cargo test --workspace --all-features`, and
  `cargo run --release -p radia-demo`.

## Submission receipt

- Submission ID: `1096888`.
- Status: `Submitted`.
- Challenge: `openai` / OpenAI Build Week.
- Project: `agent-enhanced-projects-aeps`.
- Submitted at: `2026-07-21T15:34:11.134-04:00` / 19:34:11 UTC.
- URL: `https://devpost.com/software/agent-enhanced-projects-aeps`.

Repository access status on 2026-07-21:

- `testing@devpost.com` resolved through GitHub to `devposttesting`; invitation
  `326466215` is pending with the personal-repository `write` permission GitHub
  requires for collaborators.
- `build-week-event@openai.com` has pending invitation `326467553` with the
  personal-repository `WRITE` permission. GitHub GraphQL verified the exact
  email because the REST list redacts email-only invitees.

The live deadline remained 2026-07-22 00:00 UTC (July 21 at 5:00 PM Pacific
Time). Post-submit readback confirmed the OpenAI Build Week `submitted_at`
timestamp above.
