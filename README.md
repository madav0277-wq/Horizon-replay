# Horizon Replay

A native replay-mod project for **Horizon-Android / Minecraft 26.3**.

## Current implementation

The first working-code milestone is a **camera-path recorder and playback prototype** implemented against Horizon's native trait-mod API:

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
| F10 | Stop |

These are keyboard controls in this first milestone. Touch buttons are not implemented yet.

## Important limitations

This is **not yet a complete Replay Mod**. The current code records the view camera pose, not a full player/entity/world snapshot. It does not yet record nearby entities, entity animations, block changes, inventory state, or world restoration. It also has not yet been compiled or tested inside Horizon on an Android device.

The camera hook is documented by the target source as a cinematic/replay-camera hook. The mod uses the supported `Hooks::camera_pose`, `Hooks::frame`, `Hooks::key`, and `Hooks::hud` trait methods instead of guessed game internals.

## Project layout

```
horizon-mixin.toml    Native trait-mod manifest
api/Cargo.toml        Trait-mod crate metadata
api/src/lib.rs        Recorder, playback, HUD and replay-file codec
.gitignore
README.md
```

The Android native build pipeline reads the `[api]` path from `horizon-mixin.toml`, copies `api/src`, and links the trait mod into the generated game build. The pipeline replaces the game-source revision with the exact source commit selected for that build.

## Build and installation

Horizon-Android native mods are compiled by Horizon's native build service; the phone does not compile the full game locally. A distributable `.hnmod` still needs to be packaged with the exact Horizon source revision and hook compatibility metadata for the target build. This repository currently contains the source bundle, not a tested installable package.

## Next milestones

- [ ] Compile the trait mod against the exact Horizon build selected by the Android build service.
- [ ] Test recording, saving, loading and playback on-device.
- [ ] Add touch-friendly controls.
- [ ] Add entity snapshots and replay actors.
- [ ] Add entity animation/state capture where the API permits.
- [ ] Add block/world-change capture and a safe playback representation.
- [ ] Add timeline, seeking and speed controls.
- [ ] Export/import multiple replay files.

## Compatibility notes

The implementation was inspected against the public `GoyDevv/Horizon-Android` source and its native hook catalog. The build service rewrites the game revision at build time. Do not treat this source-only repository as a finished mod release until an Android build succeeds.

## License

No license has been selected yet. Until one is added, all rights are reserved by the repository owner.
