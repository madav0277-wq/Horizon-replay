# Horizon Replay

A native camera-replay mod for **Horizon-Android / Minecraft 26.3**.

## Current implementation

The first milestone records and replays the camera path:

- Capture camera position, yaw and pitch through Horizon's `camera_pose` hook.
- Record at 20 samples per second.
- Save and load a bounded, versioned `.hrp` replay file in the mod config directory.
- Interpolate position and rotation during playback.
- Pause, stop and change playback speed.
- Show status and controls in the HUD.

The implementation uses the actual Horizon API: `Hooks::frame`, `Hooks::key`, `Hooks::hud`, `Hooks::camera_pose`, and `Host::camera/config_dir/text`. This was checked against the target Android source tree. It has **not yet been compiled or device-tested**, so it is not being presented as a verified working release.

## Native bundle format

This is a native code mod, not a content-only pack. Horizon's native source bundle format is **`.hnmod`**; `.hmod` is the content-mod format in the target documentation. The Android documentation lists the native runtime for ARM64 and x86_64 builds.

The source bundle is laid out under `mixins/horizon-replay/`. Run:

```sh
python packaging/package_hnmod.py
```

This creates `dist/horizon-replay.hnmod`. The GitHub Actions workflow also creates it as an artifact on pushes to `main`. Packaging the source archive is not the same as compiling it into the game; the Horizon launcher performs the native build for the installed game revision.

## Not implemented yet

- Entity recording and playback (players, mobs, equipment and animations).
- Block/world-change recording.
- Touch-first replay controls and a seekable timeline.
- Multiple replay files and import/export management.

## Target

- Android fork: `GoyDevv/Horizon-Android`
- Game: Minecraft 26.3
- Source revision used for API inspection: `ae4c85dc5b31da2078ee1e1009a53ced7d7ed1ba`

## Verification status

- [x] Confirmed the camera-pose, frame, key and HUD hooks exist in the target API.
- [x] Added the native bundle manifest and source layout.
- [x] Added repeatable `.hnmod` packaging and a GitHub Actions artifact workflow.
- [ ] Rust compile against the exact Android source revision.
- [ ] Native launcher build on Android.
- [ ] In-game recording/playback test on a phone.

Do not treat the archive as a tested release until the remaining checks pass.
