use egui_commonmark::*;

use std::{
    f32::consts::{FRAC_PI_4, PI},
    path::PathBuf,
};

use bevy::{prelude::*, render::view::RenderLayers};
use bevy_egui::{
    EguiContexts, EguiGlobalSettings, EguiPlugin, PrimaryEguiContext,
    egui::{self, Area, Color32, Context, Id, Modal, Response, Ui},
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use egui_material_icons::icons::{
    ICON_CENTER_FOCUS_WEAK, ICON_CHECK, ICON_EAST, ICON_NORTH, ICON_NORTH_EAST, ICON_NORTH_WEST,
    ICON_SOUTH, ICON_SOUTH_EAST, ICON_SOUTH_WEST, ICON_WEST,
};
use executor::wasmtime;

use crate::{
    app_builder::{AppType, BotConfigWrapper},
    runner::{BotExecutionData, run_bot_from_file},
    track::Track,
    ui_runner::runner_gui_setup,
    ui_test::test_gui_setup,
};

fn common_gui_setup(app: &mut App) {
    app.add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup_egui)
        .add_systems(Startup, egui_style_setup.after(setup_egui))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)));
}

fn setup_egui(mut commands: Commands, mut egui_global_settings: ResMut<EguiGlobalSettings>) {
    // Disable the automatic creation of a primary context to set it up manually for the camera we need.
    egui_global_settings.auto_create_primary_context = false;

    // Egui camera.
    commands.spawn((
        // The `PrimaryEguiContext` component requires everything needed to render a primary context.
        PrimaryEguiContext,
        Camera2d,
        // Setting RenderLayers to none makes sure we won't render anything apart from the UI.
        RenderLayers::none(),
        Camera {
            order: 1,
            ..default()
        },
    ));
}

pub struct GuiSetupPlugin {
    app_type: AppType,
}

impl GuiSetupPlugin {
    pub fn new(app_type: AppType) -> Self {
        Self { app_type }
    }
}

fn egui_style_setup(mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;
    ctx.style_mut(|style| style.visuals.panel_fill = Color32::from_rgba_unmultiplied(0, 0, 0, 0));
    egui_material_icons::initialize(ctx);
    Ok(())
}

impl Plugin for GuiSetupPlugin {
    fn build(&self, app: &mut App) {
        let has_visualization = self.app_type.has_visualization();
        let has_physics = self.app_type.has_physics();
        let (configuration, visualizer_data) = self.app_type.into_app_data();

        configuration.map(|config| app.insert_resource(BotConfigWrapper::new(config)));

        if has_visualization {
            app.add_plugins(CameraSetupPlugin);
            app.add_plugins(common_gui_setup);

            if has_physics {
                app.add_plugins(test_gui_setup);
            } else {
                let visualizer_data =
                    visualizer_data.expect("cannot build visualizer without initial data");
                runner_gui_setup(app, visualizer_data);
            }
        }
    }
}

pub fn keyboard_camera_control(
    po_camera: &mut PanOrbitCamera,
    po_transform: &Transform,
    keys: &ButtonInput<KeyCode>,
    t_delta: f32,
) -> bool {
    let up = keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let alt = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);

    const ROT_SPEED: f32 = PI / 2.0; // rad/s
    const PAN_SPEED: f32 = 1.0; // m/s
    if shift && ctrl {
        // pan on absolute Z
        if up {
            po_camera.target_focus.z += PAN_SPEED * t_delta;
        } else if down {
            po_camera.target_focus.z -= PAN_SPEED * t_delta;
        }
    } else if ctrl {
        // orbit around target point
        let yaw = if right {
            -ROT_SPEED * t_delta
        } else if left {
            ROT_SPEED * t_delta
        } else {
            0.0
        };
        let pitch = if up {
            ROT_SPEED * t_delta
        } else if down {
            -ROT_SPEED * t_delta
        } else {
            0.0
        };
        po_camera.target_yaw += yaw;
        po_camera.target_pitch += pitch;
    } else if shift {
        // move along local side and forward axes (so forward is a "zoom")
        let side_dir = po_transform.rotation.mul_vec3(Vec3::NEG_X);
        let fwd_speed = if up {
            PAN_SPEED * 2.5
        } else if down {
            -PAN_SPEED * 2.5
        } else {
            0.0
        };
        let side_speed = if left {
            PAN_SPEED
        } else if right {
            -PAN_SPEED
        } else {
            0.0
        };
        po_camera.target_focus += side_speed * t_delta * side_dir;
        po_camera.target_radius -= fwd_speed * t_delta;
        po_camera.target_radius = po_camera.target_radius.max(0.1);
    } else if alt {
        // move along local side and "flat" forward axes
        let fwd_dir = po_transform
            .rotation
            .mul_vec3(Vec3::NEG_Z)
            .with_z(0.0)
            .normalize();
        let side_dir = po_transform.rotation.mul_vec3(Vec3::NEG_X);
        let fwd_speed = if up {
            PAN_SPEED
        } else if down {
            -PAN_SPEED
        } else {
            0.0
        };
        let side_speed = if left {
            PAN_SPEED
        } else if right {
            -PAN_SPEED
        } else {
            0.0
        };
        po_camera.target_focus += side_speed * t_delta * side_dir;
        po_camera.target_focus += fwd_speed * t_delta * fwd_dir;
    }

    shift || ctrl || alt
}

pub fn rl(ui: &mut Ui, text: impl Into<String>, size: f32) -> Response {
    ui.label(egui::RichText::new(text).size(size))
}

pub fn rlc(ui: &mut Ui, text: impl Into<String>, size: f32, color: Color32) -> Response {
    ui.label(egui::RichText::new(text).size(size).color(color))
}

pub fn icon_button(ui: &mut Ui, icon: &str, size: f32) -> Response {
    ui.label(egui::RichText::new(icon).size(size))
}

enum CameraQuadrant {
    NW,
    N,
    NE,
    W,
    C,
    E,
    SE,
    S,
    SW,
}

impl CameraQuadrant {
    fn icon(&self) -> &'static str {
        match self {
            CameraQuadrant::NW => ICON_NORTH_WEST,
            CameraQuadrant::N => ICON_NORTH,
            CameraQuadrant::NE => ICON_NORTH_EAST,
            CameraQuadrant::W => ICON_WEST,
            CameraQuadrant::C => ICON_CENTER_FOCUS_WEAK,
            CameraQuadrant::E => ICON_EAST,
            CameraQuadrant::SE => ICON_SOUTH_EAST,
            CameraQuadrant::S => ICON_SOUTH,
            CameraQuadrant::SW => ICON_SOUTH_WEST,
        }
    }

    fn pitch(&self) -> f32 {
        match self {
            CameraQuadrant::C => -0.01,
            _ => -PI * 0.25,
        }
    }

    fn yaw(&self) -> f32 {
        match self {
            CameraQuadrant::NW => PI * 0.75,
            CameraQuadrant::N => PI,
            CameraQuadrant::NE => PI * 1.25,
            CameraQuadrant::W => PI * 0.5,
            CameraQuadrant::C => 0.0,
            CameraQuadrant::E => PI * 1.5,
            CameraQuadrant::SE => PI * 1.75,
            CameraQuadrant::S => 0.0,
            CameraQuadrant::SW => PI * 0.25,
        }
    }
}

fn reset_camera(po_camera: &mut PanOrbitCamera, quadrant: &CameraQuadrant, track: &Track) {
    po_camera.target_focus = track.camera_target();
    po_camera.target_yaw = quadrant.yaw();
    po_camera.target_pitch = quadrant.pitch();
    po_camera.target_radius = track.camera_radius();
    po_camera.force_update;
}

fn key_triggers_quadrant(keys: &ButtonInput<KeyCode>, quadrant: &CameraQuadrant) -> bool {
    match quadrant {
        CameraQuadrant::NW => keys.pressed(KeyCode::Numpad7),
        CameraQuadrant::N => keys.pressed(KeyCode::Numpad8),
        CameraQuadrant::NE => keys.pressed(KeyCode::Numpad9),
        CameraQuadrant::W => keys.pressed(KeyCode::Numpad4),
        CameraQuadrant::C => keys.pressed(KeyCode::Numpad5),
        CameraQuadrant::E => keys.pressed(KeyCode::Numpad6),
        CameraQuadrant::SW => keys.pressed(KeyCode::Numpad1),
        CameraQuadrant::S => keys.pressed(KeyCode::Numpad2),
        CameraQuadrant::SE => keys.pressed(KeyCode::Numpad3),
    }
}

pub fn camera_buttons(
    ui: &mut Ui,
    base_text_size: f32,
    po_camera: &mut PanOrbitCamera,
    keys: &ButtonInput<KeyCode>,
    track: &Track,
) {
    let cb_size = base_text_size * 3.0;
    ui.vertical_centered(|ui| {
        rl(ui, "Camera views", base_text_size);
        ui.separator();
        egui::Grid::new("camera_controls").show(ui, |ui| {
            for q in &[CameraQuadrant::NW, CameraQuadrant::N, CameraQuadrant::NE] {
                if icon_button(ui, q.icon(), cb_size).clicked() || key_triggers_quadrant(keys, q) {
                    reset_camera(po_camera, q, track);
                }
            }
            ui.end_row();
            for q in &[CameraQuadrant::W, CameraQuadrant::C, CameraQuadrant::E] {
                if icon_button(ui, q.icon(), cb_size).clicked() || key_triggers_quadrant(keys, q) {
                    reset_camera(po_camera, q, track);
                }
            }
            ui.end_row();
            for q in &[CameraQuadrant::SW, CameraQuadrant::S, CameraQuadrant::SE] {
                if icon_button(ui, q.icon(), cb_size).clicked() || key_triggers_quadrant(keys, q) {
                    reset_camera(po_camera, q, track);
                }
            }
            ui.end_row();
        });
    });
}

pub fn process_new_bot(
    path: PathBuf,
    output: Option<String>,
    logs: bool,
    track: Track,
    total_simulation_time_us: u32,
    period: u32,
    start_time: u32,
    sender: std::sync::mpsc::Sender<wasmtime::Result<BotExecutionData>>,
) {
    let input = path.display().to_string();
    std::thread::spawn(move || {
        sender
            .send(run_bot_from_file(
                input,
                output,
                logs,
                total_simulation_time_us,
                period,
                start_time,
                track,
            ))
            .ok();
    });
}

pub fn error_dialog(ui: &mut Ui, error_message: &mut Option<String>, base_text_size: f32) {
    let close = if let Some(msg) = &error_message {
        let modal = Modal::new(Id::new("Modal Error")).show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                rl(ui, "Error executing robot", base_text_size * 3.0);

                ui.add_space(8.0);

                rl(ui, msg, base_text_size * 2.0);

                ui.add_space(8.0);

                if icon_button(ui, ICON_CHECK, base_text_size * 4.0).clicked() {
                    ui.close();
                }
            })
        });

        modal.should_close()
    } else {
        false
    };

    if close {
        *error_message = None;
    }
}

const HELP_TEXT: &str = "
# Commands help

## Misc

| **Keypress** | **Icon** | **Command** |
| ------------ | -------- | ----------- |
| `/`  `f1`    | \u{e8fd} | help |
| `Q`          | \u{e879} | exit |
|              | \u{e8ff} | zoom in ui |
|              | \u{e900} | zoom out ui |

## Camera

| **Keypress** | **Icon** | **Command** |
| ------------ | -------- | ----------- |
| `Num7` ... `Num5` ... `Num3` | \u{f1e2} ... \u{e3b5} ... \u{f1e4} | reset camera in directions |
| `Q`          | \u{e879} | exit |
| `ctrl` + `arrow keys` / `W`  `A`  `S`  `D` | | orbit (rotate around) |
| `shift` + `arrow keys` / `W`  `A`  `S`  `D` | | pan (zoom and side) |
| `alt` + `arrow keys` / `W`  `A`  `S`  `D` | | move on x/y axis |
| `ctrl` + `shift` + `up` / `down` / `W`  `S` | | move on z axis |

## Bot

> *tip: multiple bot visualization is sorted for time ranking (best time up!)*

| **Keypress** | **Icon** | **Command** |
| ------------ | -------- | ----------- |
|              | \u{e145} | add bot (load .wasm from disk) |
|              | *click on bot name* + \u{e92e} | remove bot visualization |

#### test only:

| **Keypress** | **Icon** | **Command** |
| ------------ | -------- | ----------- |
| `arrow keys` / `W`  `A`  `S`  `D` | | move bot forward/side |
|              | *PWM sliders* | adjust pwms for forward/side commands |

## Record player

| **Keypress** | **Icon** | **Command** |
| ------------ | -------- | ----------- |
| `space`      | \u{e037} \u{e034} | play / pause |
| `home`       | \u{e045} | restart |
| `end`        | \u{e044} | go to end |
| `,`  `PgDown` | \u{e020} | go back *1 sec* |
| `.`  `PgUp`  | \u{e01f} | go forward *1 sec* |
| `ctrl` / `shift` + `,`  `PgDown` | `ctrl` / `shift` + \u{e020} | slow rewind |
| `ctrl` / `shift` + `.`  `PgUp`   | `ctrl` / `shift` + \u{e01f} | slow forward |
| `ctrl` + `shift` + `,`  `PgDown` | | go back *1 tick* |
| `ctrl` + `shift` + `.`  `PgUp`   | | go forward *1 tick* |
";

pub struct HelpState {
    pub is_open: bool,
    pub cache: CommonMarkCache,
}

impl HelpState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            cache: CommonMarkCache::default(),
        }
    }
}

pub fn help_dialog(ctx: &Context, help_state: &mut HelpState, base_text_size: f32) {
    let size = ctx.available_rect().size();

    let close = if help_state.is_open {
        let modal = Modal::new(Id::new("Modal Error")).show(ctx, |ui| {
            ui.vertical_centered(|ui: &mut Ui| {
                egui::ScrollArea::both()
                    .max_width(size.x * 0.75)
                    .max_height(size.y * 0.75)
                    .show(ui, |ui| {
                        let cmw = CommonMarkViewer::new();
                        cmw.show(ui, &mut help_state.cache, HELP_TEXT);
                    });

                if icon_button(ui, ICON_CHECK, base_text_size * 4.0).clicked() {
                    ui.close();
                }
            })
        });

        modal.should_close()
    } else {
        false
    };

    if close {
        help_state.is_open = false;
    }
}

fn setup_camera(mut commands: Commands, track: Res<Track>) {
    // Camera
    commands.spawn((PanOrbitCamera {
        focus: track.camera_target(),
        target_focus: track.camera_target(),
        yaw: Some(CameraQuadrant::C.yaw()),
        target_yaw: CameraQuadrant::C.yaw(),
        pitch: Some(CameraQuadrant::C.pitch()),
        target_pitch: CameraQuadrant::C.pitch(),
        radius: Some(track.camera_radius()),
        target_radius: track.camera_radius(),
        force_update: true,
        axis: [Vec3::X, -Vec3::Z, -Vec3::Y],
        ..Default::default()
    },));

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 0.0, 10.0),
            rotation: Quat::from_axis_angle(Vec3::new(1.0, 1.0, 0.0), FRAC_PI_4),
            ..default()
        },
    ));
}

struct CameraSetupPlugin;

impl Plugin for CameraSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PanOrbitCameraPlugin,
            // debug only:
            // bevy_rapier3d::render::RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, setup_camera)
        // Background color
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)));
    }
}
