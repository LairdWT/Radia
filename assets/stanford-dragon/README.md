# Stanford Chinese Dragon derived field

`dragon-128.rduf` is a deterministic 128 x 128 x 128 float32 unsigned-distance
volume derived from the Chinese Dragon mesh distributed by Morgan McGuire's
[Computer Graphics Archive](https://casual-effects.com/data). McGuire identifies
that OBJ as a conversion of a Stanford 3D Scanning Repository dragon scan.

## Provenance

- Original scan: Copyright 1996 Stanford University Computer Graphics
  Laboratory.
- OBJ conversion and archive: Morgan McGuire, Computer Graphics Archive, July
  2017.
- Source page:
  `https://casual-effects.com/g3d/data10/research/model/dragon/info.js`
- Source archive:
  `https://casual-effects.com/g3d/data10/research/model/dragon/dragon.zip`
- Source archive SHA-256:
  `111124359a31e4d6b2eeb5398e5bc96a5d9e2d2a130afb10bb4cfbc011bdb797`
- Expanded `dragon.obj` SHA-256:
  `aaf8d5b5196a821625f3e6b375366d61983ad66f41ab90c722f4268ced32ca3d`
- Derived `dragon-128.rduf` SHA-256:
  `9a8babdacdab6dbc3b8789b5008bbbaee4c58c7ffea42183ada83397d5cb3862`

The source contains 435,545 vertices and 871,306 triangles. The baker scales
the largest source extent to 2.8 meters, centers X/Z, places the lowest Y at
zero, adds a three-voxel volume margin, builds a nearest-triangle BVH, and
writes little-endian float32 distances. The runtime header records the volume
bounds and half-cell-diagonal interpolation error.

Reproduce and verify the artifact without adding a project dependency:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bake-dragon.ps1
```

## License boundary

This derived asset is not Apache-2.0. Stanford's repository welcomes
attributed research use and free mirroring or redistribution, but says the
models and images may not be used commercially or in a product for sale
without Stanford's permission. See the authoritative
[Stanford 3D Scanning Repository terms](https://graphics.stanford.edu/data/3Dscanrep/).

The dragon is a symbol of Chinese culture. Stanford asks users to consider the
cultural significance of repository artifacts and avoid inappropriate uses.
Radia presents a respectful rigid turntable view; it does not deform, morph,
break, or otherwise alter the dragon geometry.

Radia's Rust/WGSL code remains Apache-2.0. Commercial users must replace this
artifact with a suitably licensed mesh-derived field or obtain Stanford's
permission.
