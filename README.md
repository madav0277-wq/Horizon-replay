# Horizon Replay

A replay-mod research project for **Horizon-Android / Minecraft 26.3**.

## Important: Android support blocker

The target Android fork revision used by this project is
`ae4c85dc5b31da2078ee1e1009a53ced7d7ed1ba`. Its own release notes
(`docs/android-0.7.5.md`) state that source-only `.hnmod` entries are visible
in the catalog with an **unsupported** status, and that compiling and running
them on Android is deferred.

The fork does contain a native hook system and a `Hooks` trait API, but that
does not mean the Android launcher can currently build or load a native bundle.
A ZIP file renamed to `.hnmod` does not change that.

## Current repository status

**Prototype only. Not installable.**

The existing `api/src/lib.rs` contains an experimental camera-path recorder,
binary replay codec, interpolation and keyboard-control design. Its imports
and hook signatures have not been verified by compiling against the exact
target revision. Do not treat it as compiled or working code.

The previous packaging metadata and script did not match the official Horizon
native source-mixin bundle layout. Those unverified files and the workflow that
produced a misleading archive have been removed.

## Intended replay features

- Record camera position, yaw and pitch with timestamps.
- Save and load replay files.
- Smoothly interpolate camera movement during playback.
- Pause, resume, stop and adjust playback speed.
- Add touch-friendly controls and a timeline.
- Later investigate entity snapshots, animations and world-change capture.

## What must happen before Android installation is possible

1. Horizon-Android must add native `.hnmod` compilation/loading support, or expose
   another supported Android extension API capable of controlling the camera.
2. The replay implementation must be compiled against that exact API and fixed
   until its tests pass.
3. The bundle must pass the official source-mixin validation/build.
4. The built game must be tested on an Android device.

Until those conditions are met, there is no installable replay mod here.
