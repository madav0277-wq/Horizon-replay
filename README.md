# Horizon Replay

A replay-mod research project for **Horizon-Android / Minecraft 26.3**.

## Important: Android support blocker

The target Android fork revision used by this project is
`ae4c85dc5b31da2078ee1e1009a53ced7d7ed1ba`. Its own release notes
(`docs/android-0.7.5.md`) state that source-only `.hnmod` entries are visible
in the catalog with an **unsupported** status, and that compiling and running
them on Android is deferred.

That means this repository cannot currently produce a playable native replay
mod for that Android build. A ZIP file renamed to `.hnmod` does not change
that, and a successful archive check is not a game build.

## Current repository status

<badge>Prototype only</badge>

The existing `api/src/lib.rs` contains an experimental camera-path recorder,
binary replay codec, interpolation and keyboard-control design. It was written
against a guessed trait API and has **not** been verified against the target
Android fork. Do not treat it as compiled or working code.

The previous packaging metadata and script also do not match the official
Horizon native source-mixin bundle layout. They are retained in the repository
for audit/history only and must not be used to install the mod.

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
2. The replay implementation must be rewritten against that real API.
3. The resulting bundle must pass the official source-mixin validation/build.
4. The built game must be tested on an Android device.

Until those conditions are met, there is no installable replay mod here.
