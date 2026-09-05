# Toolkit audit — 5 September 2026

The original review had no named migration finding. This audit switched the
configuration and texture manifest in `src/data.rs` to labeled toolkit loading.
The content registry already uses that API; typed content validation remains
local.

`src/state.rs` uses toolkit versioned slots and a schema-validation callback.
Replay serialization and stable state hashes serve deterministic verification,
not a second production loader or RNG. Fixed simulation ticks, fluid/device
rules and mission admission remain game-owned.

`src/game.rs` uses AssetManager and shared render3d ray/AABB picking. Its
FoundationCamera defines orthographic framing, quarter-turn yaw, map bounds
and viewport-dependent field size. The toolkit IsometricCamera currently
defines perspective framing and a different position/distance convention;
a direct substitution would change authored views and picking. This established
camera remains local, using shared picking with its resulting Camera3D.

The UI uses VirtualUi, Pointer, shared text and surface helpers; main uses the
toolkit capture lifecycle. No local generic content loader, storage backend,
audio bank, random stream or particle engine was found.

Also fixed four unrelated Clippy findings: three parity expressions now use
`is_multiple_of`, and a constant verification notice uses String::from.

Final validation: 76 checks including deterministic replay and save validation,
formatting, strict all-target/all-feature Clippy and Rust source-size limits.
Default `publish.ps1` passed Windows/WebGL release builds, Preview deployment
and Project Roost tracking. Content is embedded and terrain is procedural;
the publisher correctly registered zero external runtime assets.
