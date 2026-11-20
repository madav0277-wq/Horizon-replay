# Horizon Replay

A native camera-replay mod for **Horizon-Android / Minecraft 26.3**.

## What is implemented

- Records camera position, yaw and pitch through Horizon's `camera_pose` hook.
- Samples at 20 times per second.
- Saves and loads a bounded, versioned `.hrp` replay file.
- Replays the camera path with interpolated position and rotation.
- Supports pause, stop and playback speed changes.
- Displays recording/playback status in the HUD.

Keyboard controls in this prototype:

| Key | Action |
|---|---|
| F8 | Start recording / stop and save |
| F9 | Load and play latest replay |
| F7 | Pause / resume |
| F5 / F6 | Decrease / increase playback speed |
| F10 | Stop playback |

The current milestone is camera-only. Entity snapshots, animations, block/world changes, touch-first controls, timeline seeking and replay library management are not implemented yet.

## Native mod format

The target Android branch documents native Rust mods as **`.hnmod`** bundles. A content-only `.hmod` is not interchangeable with a native replay mod.

This repository follows the fork's native bundle layout:
- `horizon-native.json`
- `mixins/horizon-replay/horizon-mixin.toml`
- `mixins/horizon-replay/api/Cargo.toml`
- `mixins/horizon-replay/api/src/lib.rs`

The metadata declares native hook API version 415. The source uses the documented `Hooks::frame`, `Hooks::key`, `Hooks::hud`, `Hooks::camera_pose`, and `Host::camera/config_dir/text` APIs present in the pinned Android source revision.

Run `python packaging/package_hnmod.py` to create `dist/horizon-replay.hnmod`. GitHub Actions runs a Rust compile/test against the exact Android source revision before packaging the archive.

## Verification status

- [x] Confirmed the camera-pose, frame, key and HUD hooks exist in the target API.
- [x] Confirmed the host camera, config-directory and HUD text methods exist.
- [x] Added the native bundle manifest and source layout.
- [x] Added a repeatable `.hnmod` packager.
- [ ] CI compile and unit tests against the pinned Android API.
- [ ] Native build using the Android launcher for a real game instance.
- [ ] In-game recording/playback test on an Android device.

A successful ZIP/package step alone does not prove the mod works in-game. Do not treat it as a tested release until the remaining checks pass.
