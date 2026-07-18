# RADIA MVP visual evidence

Purpose: acceptance. Source: direct WGPU render attachment readback. Revision:
`25278ed1a5e0a18161d635fcfdd5ba90c0487f4f`.

`run-1` and `run-2` contain independent fixed-state off/on captures and the
project adapter's TSV report. Both RADIA captures have SHA-256
`889628a04fa663f200d79bdc06a70ce36ab1fd53dab551f99266df71eac40442`.

`determinism-manifest.json` compares the two RADIA runs exactly. Its AEP report
passes with zero differing decoded samples. `controlled-delta-manifest.json`
compares off and on captures without suppressing differences; the project TSV
applies the acceptance rule to the receiver ROI and records a `100/255` maximum
channel delta against the required `4/255` threshold.

Reproduce:

```powershell
$env:RADIA_ADAPTER_NAME = 'NVIDIA'
cargo run -p radia-demo -- evidence --width 320 --height 180 --samples 1024 --output-dir Temp\evidence\run-1
cargo run -p radia-demo -- evidence --width 320 --height 180 --samples 1024 --output-dir Temp\evidence\run-2
agent-code-skills image validate docs\evidence\radia-mvp-rtx4070\controlled-delta-manifest.json
agent-code-skills image validate docs\evidence\radia-mvp-rtx4070\determinism-manifest.json
```

The comparison contact sheets are diagnostic renderings. Raw PNGs and JSON/TSV
reports remain authoritative.
