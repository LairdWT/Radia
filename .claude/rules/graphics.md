---
paths:
  - "**/*.wgsl"
---

Before writing graphics code here, read plugins/graphics/CONVENTIONS.md
from the agent-enhanced-project library (or the vendored copy under
docs/conventions/). CODE.md deviations override the referenced
conventions. Every lint suppression carries a reason and a tracking ref.

Load `graphics:wgpu-core`, then `graphics:wgsl-core`, and only their smallest
routed spokes. Audit changed graphics code with the graphics-audit skill.
