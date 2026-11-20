# Horizon Replay

A native replay-mod project for **Horizon-Android / Minecraft 26.3**.

## Package format

This is a native Rust mod and is packaged as **`.hnmod`**. It is not an APK and it is not a content-only `.hmod`.

## Current implementation

The first implementation milestone is a camera-path recorder and playback prototype using Horizon's native trait-mod API:

- Records camera position, yaw and pitch at up to 20 samples per second.
- Saves a bounded replay file as `latest.hrp` under Horizon's mod config directory.
- Loads and plays the saved path with linear position interpolation and shortest-path yaw interpolation.
- Overrides the camera pose during playback.
- Displays a small HUD status line.
- Uses a compact, versioned binary format with bounds checks and codec tests.

### Controls

| Key | Action |
|---|---|
| F8 | Start recording; press again to save |
| F9 | Load and play the latest replay |
| F7 | Pause / resume playback |
| F5 | Halve playback speed (minimum 0.25x) |
| F6 | Double playback speed (maximum 4x) |
| F10 | Stop |

These are keyboard controls in this first milestone. Touch buttons are not implemented yet.

## Build the .hnmod bundle

The repository includes `horizon-native.json` and a reproducible packager:

```sh
python3 tools/package_hnmod.py
```

It creates `dist/horizon_replay.hnmod`. GitHub Actions also runs this packager on pushes to `main` and uploads the `.hnmod` as the `horizon-replay-hnmod` workflow artifact.

The package pins Horizon commit `ae4c85dc5b31da2078ee1e1009a53ced7d7ed1ba`, horizon-mixins commit `1e758beab4f00b7ab82183f5b4a7ec4f42599a98`, and native hooks API 1 (the `camera_pose` contract). Horizon's launcher builds native code for the game instance; the `.hnmod` is the source bundle, not a precompiled APK or shared library.

## Important limitations

This is **not yet a complete Replay Mod**. The current code records the view camera pose, not a full player/entity/world snapshot. It does not yet record nearby entities, entity animations, block changes, inventory state, or world restoration.

The code and package have not yet been compiled by Horizon's native build pipeline or tested inside the Android game. A successful ZIP/package check is not a successful native compilation.

## Project layout

```
horizon-native.json     Native bundle metadata
horizon-mixin.toml      Native mod manifest and Horizon revision pin
api/Cargo.toml          Trait-mod crate metadata
api/src/lib.rs          Recorder, playback, HUD and replay-file codec
tools/package_hnmod.py  .hnmod packager and archive validation
.github/workflows/      Automatic .hnmod packaging
```

## Next milestones

- [ ] Compile the native bundle against the pinned Horizon source.
- [ ] Test recording, saving, loading and playback on-device.
- [ ] Add touch-friendly controls.
- [ ] Add entity snapshots and replay actors.
- [ ] Add entity animation/state capture where the API permits.
- [ ] Add block/world-change capture and a safe playback representation.
- [ ] Add a touch-friendly timeline and seeking controls.
- [ ] Export/import multiple replay files.

## License

No license has been selected yet. Until one is added, all rights are reserved by the repository owner.
