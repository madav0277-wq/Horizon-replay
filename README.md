# Horizon Replay

A replay-mod project for **Minecraft Rust Edition (Horizon)** on Android.

> **Development status:** Early project setup. Recording, playback, camera controls, and replay-file support are planned, not yet implemented. Compatibility and available mod APIs still need to be verified against the exact Horizon source and SDK.

## Project goals

- Record the local player's position, rotation, and game tick over time.
- Investigate recording nearby entities and relevant world updates where the supported API permits.
- Replay recorded movement with smooth interpolation.
- Provide replay controls such as play, pause, stop, seek, and playback speed.
- Explore a touch-friendly HUD and detached replay camera.
- Keep replay files compact and suitable for mobile storage and memory limits.

These are goals, not a claim that the features currently work.

## Compatibility

The target is Horizon / Horizon-Android. The exact game version, mod loader interface, SDK version, supported hooks, and packaging format must be confirmed from the upstream source before implementation.

## Development

No build or installation instructions are published yet because the project structure and supported Horizon mod API have not been verified. Once those are confirmed, this section will document the tested toolchain and exact build steps.

## Current status

- [x] Repository initialized
- [x] Basic project documentation
- [ ] Inspect and pin the upstream Horizon API/SDK
- [ ] Create the mod skeleton using verified APIs
- [ ] Implement recording
- [ ] Implement playback and interpolation
- [ ] Add touch-friendly replay controls
- [ ] Test on Android

## Contributing

The project is at an early stage. Please open an issue before building against assumed APIs or submitting compatibility claims.

## License

No license has been selected yet. Until one is added, all rights are reserved by the repository owner.
