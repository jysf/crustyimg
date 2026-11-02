---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-062                     # stable, zero-padded within the project
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-016                      # parent project
repo:
  id: crustyimg

created_at: 2026-11-02
shipped_at: null

# What part of the project's value thesis this stage advances.
value_contribution:
  advances: >
    The "real developed pixels and truthful metadata" half of PROJ-016's
    thesis, past what STAGE-056 delivers: framing and warp become develop
    options, `info` reports the sensor facts irradiance 0.2 exposes, and the
    embedded preview can be compared with the plane it was rendered from.
  delivers:
    - "Develop options `framing` (standard crop or full active area) and `warp_factor` (0.0..=1.0), reachable from a recipe and part of the cache key"
    - "`info` reporting the full sensor, active area and default crop geometry, bits per sample, black and white level, the radial warp coefficients and the measured maximum corner displacement"
    - "A preview delta: how many pixels the camera's embedded JPEG clips at 255 against how many the sensor saturates at or above white level, with the recoverable headroom in EV"
  explicitly_does_not:
    - "Add colour RAW or a second camera"
    - "Change the default build's RAW output (STAGE-056's byte-identical criterion still binds)"
    - "Add a black-calibration mode or an uncropped-overscan mode. The first handoff listed both and the updated one dropped them; they return only if irradiance 0.2 exposes them"
    - "Add a CLI verb (the surface is frozen at 14 by STAGE-030)"
---

# STAGE-062: Sensor forensics — geometry, warp, and the preview delta

## What This Stage Is

When this stage ships, a `raw-develop` build can render the same DNG at any
framing and any warp from 0% (native, no resampling) to full rectilinear,
`info` states what the sensor actually is, and the embedded preview can be
checked against the plane it claims to show.

## Why Now

It is not yet time. It depends on STAGE-056 (the adapter, the two tiers, the
cache key) and on `irradiance` 0.2, which is unpublished. It is framed now
because the cache-key rule STAGE-056 sets, that anything changing the pixels
is part of the key, applies directly to these options, and a design that adds
them after STAGE-056 ships costs a second key migration.

## Success Criteria

- `framing = full-active-area` with `warp_factor = 0.0` produces the active-area dimensions with no resampling, asserted by comparing against irradiance's unpacked plane
- Changing `framing` or `warp_factor` misses the build cache; leaving them alone hits it
- `info` on the measured DNG prints all three geometries and the stated levels, each checked against exiftool rather than against irradiance's own output
- The preview delta is computed from a decoded preview and irradiance's histogram, and a synthetic fixture with a known number of clipped pixels returns that number
- The default build's RAW output stays byte-identical

## Scope

### In scope
- The Items in the Spec Backlog below

### Explicitly out of scope
- See `value_contribution.explicitly_does_not`

## Spec Backlog

Ordered. The second and third depend on the first only for the geometry they report.

- [ ] (not yet written) — **[M] Framing and warp as develop options.** Options on the adapter, recipe keys with refusal at load, both folded into `compute_key` next to `DEVELOP_VERSION`.
- [ ] (not yet written) — **[M] Sensor facts in `info`.** The geometry triple, levels, bit depth, the warp coefficients and the measured corner displacement. How the extra detail is requested (a flag on `info`, or always) is a design decision.
- [ ] (not yet written) — **[M] The preview delta.** Decode the embedded preview JPEG in crustyimg, take the raw ADU histogram from irradiance, report both clip counts and the headroom. The spec must define "headroom in EV" before building it.

**Count:** 0 shipped / 0 active / 3 pending

## Design Notes

**The API names are unverified.** `WarpRect::lerp`, the preview slice accessor and the caller-allocated `[u32; 65536]` histogram come from a handoff written by the irradiance side. None has been read against irradiance's source from this repo. The design cycle starts by reading them.

**The delta compares two different things.** A camera preview is tone-mapped, white-balanced output and the plane is linear sensor data, so "clipped at 255" and "at or above white level" are not the same threshold on the same scale. The spec has to say how a preview pixel maps back to a sensor value, or the EV headroom it reports is a number with no meaning. Check the claim by building a fixture whose answer is known.

**Where the figures surface is a decision.** The sensor card in PROJ-018 prints these same numbers, so they should come from one report type that `info` and the card both read.

## Dependencies

### Depends on
- STAGE-056 — the adapter, `SourceContainer::RawDeveloped`, the cache-key field
- External: `irradiance` 0.2 published (the same crates.io gate as 0.1.0)
- A Leica Q2 Monochrom DNG for the real-file checks

### Enables
- PROJ-018's sensor card and print-master stages

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
