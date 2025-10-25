use std::{f32::consts::PI, path::PathBuf, sync::Mutex};

use bevy::{prelude::*, render::view::RenderLayers};
use bevy_egui::{
    EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext,
    egui::{self, Color32, Id, Modal, Response, Stroke, Ui},
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_rapier3d::render::RapierDebugRenderPlugin;
use egui_file_dialog::FileDialog;
use egui_material_icons::icons::{
    ICON_ADD, ICON_CANCEL, ICON_CENTER_FOCUS_WEAK, ICON_CHECK, ICON_DELETE, ICON_EAST,
    ICON_EXIT_TO_APP, ICON_FAST_FORWARD, ICON_FAST_REWIND, ICON_HELP, ICON_NORTH, ICON_NORTH_EAST,
    ICON_NORTH_WEST, ICON_PAUSE, ICON_PLAY_ARROW, ICON_SKIP_NEXT, ICON_SKIP_PREVIOUS, ICON_SOUTH,
    ICON_SOUTH_EAST, ICON_SOUTH_WEST, ICON_WEST, ICON_ZOOM_IN, ICON_ZOOM_OUT,
};
use execution_data::{MotorDriversDutyCycles, PWM_MAX, SensorsData};
use executor::wasmtime;

use crate::{
    app_builder::{AppType, BotConfigWrapper, VisualizerData},
    runner::{BotExecutionData, run_bot_from_file},
    server::start_server,
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

pub fn runner_gui_setup(app: &mut App, visualizer_data: VisualizerData) {
    let gui_state = RunnerGuiState::new(
        visualizer_data.output(),
        visualizer_data.logs(),
        visualizer_data.period(),
    );

    match visualizer_data {
        VisualizerData::Server {
            address,
            port,
            period,
        } => {
            let sender = gui_state.get_bot_sender().clone();
            start_server(address, port, period, sender)
                .map_err(|err| {
                    eprintln!("error starting HTTP server: {}", err.to_string());
                    err
                })
                .expect("failed to start server");
        }
        VisualizerData::Runner { .. } => {
            // TODO: spawn bot visualizer entity
        }
    }
    app.add_systems(EguiPrimaryContextPass, runner_gui_update)
        .insert_resource(gui_state);
}

pub fn test_gui_setup(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, test_gui_update)
        .insert_resource(TestGuiState::default());
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
                    visualizer_data.expect("cannot build visualized without initial data");
                runner_gui_setup(app, visualizer_data);
            }
        }
    }
}

#[derive(Resource)]
pub struct RunnerGuiState {
    file_dialog: FileDialog,
    new_bot_sender: Mutex<std::sync::mpsc::Sender<wasmtime::Result<BotExecutionData>>>,
    new_bot_receiver: Mutex<std::sync::mpsc::Receiver<wasmtime::Result<BotExecutionData>>>,
    base_text_size: f32,
    play_time_sec: f32,
    play_active: bool,
    play_max_sec: f32,
    output: Option<String>,
    logs: bool,
    period: u32,
    bot_with_pending_remove: Option<BotName>,
}

impl RunnerGuiState {
    pub fn new(output: Option<String>, logs: bool, period: u32) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            file_dialog: FileDialog::new(),
            new_bot_sender: Mutex::new(sender),
            new_bot_receiver: Mutex::new(receiver),
            base_text_size: 10.0,
            play_time_sec: 0.0,
            play_active: false,
            play_max_sec: 60.0,
            output,
            logs,
            period,
            bot_with_pending_remove: None,
        }
    }

    pub fn get_bot_sender(&self) -> std::sync::mpsc::Sender<wasmtime::Result<BotExecutionData>> {
        self.new_bot_sender.lock().unwrap().clone()
    }

    pub fn handle_new_bots(&self) {
        while let Ok(bot) = self.new_bot_receiver.lock().unwrap().try_recv() {
            match bot {
                Ok(bot) => println!("new bot steps: len {}", bot.data.steps.len()),
                Err(err) => error!("Error receiving new bot: {}", err),
            }
        }
    }
}

fn runner_gui_update(
    mut contexts: EguiContexts,
    mut gui_state: ResMut<RunnerGuiState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut camera: Query<(&mut PanOrbitCamera, &mut Transform)>,
    time: Res<Time>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let (mut po_camera, mut po_transform) = camera.single_mut()?;

    if keyboard_input.just_released(KeyCode::KeyQ) || keyboard_input.just_released(KeyCode::Escape)
    {
        exit.write(AppExit::Success);
    }

    if gui_state.play_active {
        gui_state.play_time_sec += time.delta_secs();
    }
    gui_state.play_time_sec = gui_state.play_time_sec.min(gui_state.play_max_sec).max(0.0);

    egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(false)
        .default_height(gui_state.base_text_size * 1.8)
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let size = gui_state.base_text_size * 4.0;
                if icon_button(ui, ICON_HELP, size).clicked() {
                    println!("HELP!");
                }
                ui.separator();

                if icon_button(ui, ICON_SKIP_PREVIOUS, size).clicked() {
                    gui_state.play_time_sec = 0.0;
                }
                if gui_state.play_active {
                    if icon_button(ui, ICON_PAUSE, size).clicked() {
                        gui_state.play_active = false;
                    }
                } else {
                    if icon_button(ui, ICON_PLAY_ARROW, size).clicked() {
                        gui_state.play_active = true;
                    }
                }
                if icon_button(ui, ICON_SKIP_NEXT, size).clicked() {
                    gui_state.play_time_sec = gui_state.play_max_sec;
                }

                ui.add_space(size / 2.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icon_button(ui, ICON_ADD, size).clicked() {
                        gui_state.file_dialog.pick_file();
                    }
                    if icon_button(ui, ICON_ZOOM_IN, size).clicked() {
                        gui_state.base_text_size += 1.0;
                        gui_state.base_text_size = gui_state.base_text_size.max(3.0);
                    }
                    if icon_button(ui, ICON_ZOOM_OUT, size).clicked() {
                        gui_state.base_text_size -= 1.0;
                    }

                    let w = ui.available_width();
                    ui.style_mut().spacing.slider_width = w;
                    let max_time = gui_state.play_max_sec;
                    ui.add(
                        egui::Slider::new(&mut gui_state.play_time_sec, 0.0..=max_time)
                            .show_value(false),
                    );
                    ui.add_space(size / 2.0);
                    rl(ui, format!("{:.3}", gui_state.play_time_sec), size);
                });

                gui_state.file_dialog.update(ctx);
                if let Some(path) = gui_state.file_dialog.take_picked() {
                    let sender = gui_state.get_bot_sender();
                    let output = gui_state.output.clone();
                    let logs = gui_state.logs;
                    let period = gui_state.period;
                    std::thread::spawn(move || {
                        process_new_bot(path, output, logs, period, sender);
                    });
                }
                gui_state.handle_new_bots();
            });
        });

    let cb_size = gui_state.base_text_size * 3.0;
    egui::SidePanel::left("left_panel")
        .resizable(false)
        .default_width(cb_size * 3.0)
        .show_separator_line(false)
        .show(ctx, |ui| {
            camera_buttons(
                ui,
                gui_state.base_text_size,
                po_camera.as_mut(),
                po_transform.as_mut(),
            );
        });

    egui::SidePanel::right("right_panel")
        .resizable(false)
        .default_width(gui_state.base_text_size * 30.0)
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                rl(ui, "Robots", gui_state.base_text_size);
                ui.separator();
                ui.horizontal(|ui| {
                    if bot_status(
                        ui,
                        "Test BOT",
                        Color32::RED,
                        Color32::BLUE,
                        time.elapsed_secs(),
                        BotStatus::Running,
                        gui_state.base_text_size,
                    ) {
                        gui_state.as_mut().bot_with_pending_remove = Some(BotName {
                            name: "Test BOT".to_string(),
                            c1: Color32::RED,
                            c2: Color32::BLUE,
                        })
                    }
                });
            });

            if ask_bot_remove(ui, gui_state.as_mut()) == Some(true) {
                gui_state.as_mut().bot_with_pending_remove = None;
            }
        });

    Ok(())
}

#[derive(Resource)]
struct TestGuiState {
    base_text_size: f32,
    pwm_fwd_cmd: i16,
    pwm_side_cmd: i16,
}

impl Default for TestGuiState {
    fn default() -> Self {
        Self {
            base_text_size: 8.0,
            pwm_fwd_cmd: PWM_MAX / 2,
            pwm_side_cmd: PWM_MAX / 2,
        }
    }
}

fn test_gui_update(
    mut contexts: EguiContexts,
    mut gui_state: ResMut<TestGuiState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut pwm: ResMut<MotorDriversDutyCycles>,
    sensors: Res<SensorsData>,
    mut camera: Query<(&mut PanOrbitCamera, &mut Transform)>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let (mut po_camera, mut po_transform) = camera.single_mut()?;

    if keyboard_input.just_released(KeyCode::KeyQ) || keyboard_input.just_released(KeyCode::Escape)
    {
        exit.write(AppExit::Success);
    }

    egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(false)
        .default_height(gui_state.base_text_size * 1.8)
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let size = gui_state.base_text_size * 4.0;
                if icon_button(ui, ICON_HELP, size).clicked() {
                    println!("HELP!");
                }
                ui.separator();

                for sensor_index in 0..16 {
                    let value = (sensors.line_sensors[sensor_index] * 255.0 / 100.0) as u8;
                    rl(ui, &format!("{:3}", value), size * 0.5);
                    ui.add_space(size / 2.0);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icon_button(ui, ICON_ZOOM_IN, size).clicked() {
                        gui_state.base_text_size += 1.0;
                        gui_state.base_text_size = gui_state.base_text_size.max(3.0);
                    }
                    if icon_button(ui, ICON_ZOOM_OUT, size).clicked() {
                        gui_state.base_text_size -= 1.0;
                    }
                });
            });
        });

    let cb_size = gui_state.base_text_size * 3.0;
    egui::SidePanel::left("left_panel")
        .resizable(false)
        .default_width(cb_size * 3.0)
        .show_separator_line(false)
        .show(ctx, |ui| {
            camera_buttons(
                ui,
                gui_state.base_text_size,
                po_camera.as_mut(),
                po_transform.as_mut(),
            );
        });

    egui::SidePanel::right("right_panel")
        .resizable(false)
        .default_width(gui_state.base_text_size * 30.0)
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                rl(ui, "PWM limits", gui_state.base_text_size);
                ui.separator();

                ui.style_mut().spacing.slider_width = 250.0;
                rl(ui, "Forward", gui_state.base_text_size * 1.25);
                ui.add(egui::Slider::new(&mut gui_state.pwm_fwd_cmd, 0..=PWM_MAX).show_value(true));
                ui.add_space(4.0);
                rl(ui, "Side", gui_state.base_text_size * 1.25);
                ui.add(
                    egui::Slider::new(&mut gui_state.pwm_side_cmd, 0..=PWM_MAX).show_value(true),
                );
            });
        });

    // Handle motor control
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let forward = if up {
        1
    } else if down {
        -1
    } else {
        0
    };
    let side = if left {
        -1
    } else if right {
        1
    } else {
        0
    };

    pwm.left =
        (forward * gui_state.pwm_fwd_cmd + side * gui_state.pwm_side_cmd).clamp(-PWM_MAX, PWM_MAX);
    pwm.right =
        (forward * gui_state.pwm_fwd_cmd - side * gui_state.pwm_side_cmd).clamp(-PWM_MAX, PWM_MAX);

    Ok(())
}

fn rl(ui: &mut Ui, text: impl Into<String>, size: f32) -> Response {
    ui.label(egui::RichText::new(text).size(size))
}

fn icon_button(ui: &mut Ui, icon: &str, size: f32) -> Response {
    ui.label(egui::RichText::new(icon).size(size))
}

const CAMERA_Z: f32 = 5.0;

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

fn reset_camera(
    po_camera: &mut PanOrbitCamera,
    po_transform: &mut Transform,
    quadrant: CameraQuadrant,
) {
    po_camera.target_focus = Vec3::ZERO;
    po_camera.target_yaw = quadrant.yaw();
    po_camera.target_pitch = quadrant.pitch();
    po_camera.target_radius = CAMERA_Z;
    po_camera.force_update;
}

fn camera_buttons(
    ui: &mut Ui,
    base_text_size: f32,
    po_camera: &mut PanOrbitCamera,
    po_transform: &mut Transform,
) {
    let cb_size = base_text_size * 3.0;
    ui.vertical_centered(|ui| {
        rl(ui, "Camera views", base_text_size);
        ui.separator();
        egui::Grid::new("camera_controls").show(ui, |ui| {
            for q in [CameraQuadrant::NW, CameraQuadrant::N, CameraQuadrant::NE] {
                if icon_button(ui, q.icon(), cb_size).clicked() {
                    reset_camera(po_camera, po_transform, q);
                }
            }
            ui.end_row();
            for q in [CameraQuadrant::W, CameraQuadrant::C, CameraQuadrant::E] {
                if icon_button(ui, q.icon(), cb_size).clicked() {
                    reset_camera(po_camera, po_transform, q);
                }
            }
            ui.end_row();
            for q in [CameraQuadrant::SW, CameraQuadrant::S, CameraQuadrant::SE] {
                if icon_button(ui, q.icon(), cb_size).clicked() {
                    reset_camera(po_camera, po_transform, q);
                }
            }
            ui.end_row();
        });
    });
}

fn process_new_bot(
    path: PathBuf,
    output: Option<String>,
    logs: bool,
    period: u32,
    sender: std::sync::mpsc::Sender<wasmtime::Result<BotExecutionData>>,
) {
    let input = path.display().to_string();
    std::thread::spawn(move || {
        sender
            .send(run_bot_from_file(input, output, logs, period))
            .ok();
    });
}

fn ask_bot_remove(ui: &mut Ui, gui_state: &mut RunnerGuiState) -> Option<bool> {
    let mut response = Some(false);
    if let Some(bot_with_pending_remove) = &gui_state.bot_with_pending_remove {
        let modal = Modal::new(Id::new("Modal Remove")).show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                rl(ui, "Remove robot?", gui_state.base_text_size * 2.0);

                ui.add_space(8.0);

                bot_name(
                    ui,
                    &bot_with_pending_remove.name,
                    bot_with_pending_remove.c1,
                    bot_with_pending_remove.c2,
                    gui_state.base_text_size,
                );

                ui.add_space(8.0);

                let mut yes = false;
                let mut no = false;
                egui::Sides::new().show(
                    ui,
                    |ui| {
                        if icon_button(ui, ICON_DELETE, gui_state.base_text_size * 4.0).clicked() {
                            yes = true;
                            ui.close();
                        }
                    },
                    |ui| {
                        if icon_button(ui, ICON_CANCEL, gui_state.base_text_size * 4.0).clicked() {
                            no = true;
                            ui.close();
                        }
                    },
                );
                if yes {
                    response = Some(true);
                }
                if no {
                    response = Some(false);
                }
            })
        });

        if modal.should_close() {
            gui_state.bot_with_pending_remove = None;
        }
    }
    response
}

fn bot_name(ui: &mut Ui, name: &str, c1: Color32, c2: Color32, base_text_size: f32) -> bool {
    let val = (c1.r() + c1.g() + c1.b()) / 3;
    let color = if val < 128 {
        Color32::WHITE
    } else {
        Color32::BLACK
    };
    egui::Frame::default()
        .fill(c1)
        .stroke(Stroke {
            color: c2,
            width: 8.0,
        })
        .corner_radius(8)
        .inner_margin(8)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(name)
                    .color(color)
                    .strong()
                    .size(base_text_size * 2.0),
            )
        })
        .inner
        .clicked()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BotStatus {
    Running,
    Ended,
    Out,
}

impl BotStatus {
    pub fn color(&self) -> Color32 {
        match self {
            BotStatus::Running => Color32::WHITE,
            BotStatus::Ended => Color32::GREEN,
            BotStatus::Out => Color32::RED,
        }
    }
}

fn bot_status(
    ui: &mut Ui,
    name: &str,
    c1: Color32,
    c2: Color32,
    time_sec: f32,
    status: BotStatus,
    base_text_size: f32,
) -> bool {
    let mut response = false;
    ui.horizontal(|ui| {
        response = bot_name(ui, name, c1, c2, base_text_size);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{:.2}", time_sec))
                    .color(status.color())
                    .strong()
                    .size(base_text_size * 3.0),
            );
        });
    });
    response
}

struct BotName {
    name: String,
    c1: Color32,
    c2: Color32,
}

fn setup_camera(mut commands: Commands) {
    // Camera
    commands.spawn((PanOrbitCamera {
        focus: Vec3::ZERO,
        target_focus: Vec3::ZERO,
        yaw: Some(CameraQuadrant::C.yaw()),
        target_yaw: CameraQuadrant::C.yaw(),
        pitch: Some(CameraQuadrant::C.pitch()),
        target_pitch: CameraQuadrant::C.pitch(),
        radius: Some(CAMERA_Z),
        target_radius: CAMERA_Z,
        force_update: true,
        axis: [Vec3::X, -Vec3::Z, -Vec3::Y],
        ..Default::default()
    },));
}

struct CameraSetupPlugin;

impl Plugin for CameraSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PanOrbitCameraPlugin,
            // #FIXME: debug only
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, setup_camera)
        // Background color
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)));
    }
}
