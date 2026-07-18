---
paths:
  - "crates/radia-math/**/*.rs"
---

Load `math:math-core` first, then only the smallest routed spokes. Declare
scalar precision, units, handedness, transform direction, depth range, screen
origin, and pixel-center rule before using convention-sensitive formulas.
Language and graphics adapters own representation and API behavior.
