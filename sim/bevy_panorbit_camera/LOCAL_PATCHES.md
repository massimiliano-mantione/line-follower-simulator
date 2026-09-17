# Local patches

This is a vendored copy of the crates.io release of `bevy_panorbit_camera`,
currently **0.35.1**. Re-apply the following on every re-vendor:

- `Cargo.toml` / `Cargo.toml.orig`: `bevy_egui` requirement relaxed from `0.40`
  to `0.42`, so it matches the `bevy_egui` the simulator uses (the one that
  tracks the latest `egui`). Upstream still pins the older one.
- `src/egui.rs`: egui 0.36 renamed the `Context` methods used here —
  `wants_pointer_input` → `egui_wants_pointer_input`,
  `wants_keyboard_input` → `egui_wants_keyboard_input`,
  `is_pointer_over_area` → `is_pointer_over_egui`.
- `examples/egui.rs`: egui 0.36 replaced `SidePanel`/`TopBottomPanel` with
  `Panel`, which is shown inside a `Ui` instead of on the `Context`.
- `examples/egui_multiple_windows.rs`: bevy_egui 0.42 renamed
  `EguiMultipassSchedule` to `EguiSchedule`.

## No longer needed

The reason this crate was vendored in the first place (commit `4f32b9d`) was a
fix for panning along the wrong axis when the camera uses a non-default
`PanOrbitCamera::axis`, which the simulator does. Upstream fixed this in 0.35.1
in a way that is equivalent to our patch, so it is no longer carried here.
