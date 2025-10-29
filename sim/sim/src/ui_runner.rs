use std::sync::Mutex;

use bevy::{
    app::{App, AppExit},
    ecs::{
        event::EventWriter,
        resource::Resource,
        system::{Query, ResMut},
    },
    input::{ButtonInput, keyboard::KeyCode},
    prelude::*,
    time::Time,
    transform::components::Transform,
};
use bevy_egui::{
    EguiContexts, EguiPrimaryContextPass,
    egui::{self, Color32, Id, Modal, Stroke, Ui},
};
use bevy_panorbit_camera::PanOrbitCamera;
use egui_file_dialog::FileDialog;
use egui_material_icons::icons::{
    ICON_ADD, ICON_CANCEL, ICON_DELETE, ICON_EXIT_TO_APP, ICON_FAST_FORWARD, ICON_FAST_REWIND,
    ICON_HELP, ICON_PAUSE, ICON_PLAY_ARROW, ICON_SKIP_NEXT, ICON_SKIP_PREVIOUS, ICON_ZOOM_IN,
    ICON_ZOOM_OUT,
};
use execution_data::BotStatus;
use executor::wasmtime;

use crate::{
    app_builder::VisualizerData,
    bot::vis::BotAssets,
    runner::BotExecutionData,
    server::start_server,
    track::Track,
    ui::{
        camera_buttons, error_dialog, help_dialog, icon_button, keyboard_camera_control,
        process_new_bot, rl,
    },
    visualizer::{
        BotVisualization, spawn_bot_visualization, sync_bot_body, sync_bot_layers, sync_bot_wheel,
    },
};

pub fn runner_gui_setup(app: &mut App, visualizer_data: VisualizerData) {
    let gui_state = RunnerGuiState::new(
        visualizer_data.output(),
        visualizer_data.logs(),
        visualizer_data.period(),
        visualizer_data.start_time(),
        visualizer_data.first_bot(),
    );

    match visualizer_data {
        VisualizerData::Server {
            address,
            port,
            period,
            start_time,
        } => {
            let sender = gui_state.get_bot_sender().clone();
            let track = app.world().resource::<Track>().clone();
            start_server(address, port, track, period, start_time, sender)
                .map_err(|err| {
                    eprintln!("error starting HTTP server: {}", err.to_string());
                    err
                })
                .expect("failed to start server");
        }
        VisualizerData::Runner { .. } => {}
    }
    app.add_systems(EguiPrimaryContextPass, runner_gui_update)
        .insert_resource(gui_state);
    app.add_systems(Update, (sync_bot_layers, sync_bot_body, sync_bot_wheel));
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
    bot_count: usize,
    output: Option<String>,
    logs: bool,
    period: u32,
    start_time: u32,
    bot_with_pending_remove: Option<(Entity, BotName)>,
    error_message: Option<String>,
    help_open: bool,
}

impl RunnerGuiState {
    pub fn new(
        output: Option<String>,
        logs: bool,
        period: u32,
        start_time: u32,
        first_bot: Option<BotExecutionData>,
    ) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        if let Some(bot) = first_bot {
            sender.send(Ok(bot)).unwrap();
        }

        Self {
            file_dialog: FileDialog::new(),
            new_bot_sender: Mutex::new(sender),
            new_bot_receiver: Mutex::new(receiver),
            base_text_size: 10.0,
            play_time_sec: 0.0,
            play_active: false,
            play_max_sec: 60.0,
            bot_count: 0,
            output,
            logs,
            period,
            start_time,
            bot_with_pending_remove: None,
            error_message: None,
            help_open: false,
        }
    }

    pub fn play_time_sec(&self) -> f32 {
        self.play_time_sec
    }

    pub fn get_bot_sender(&self) -> std::sync::mpsc::Sender<wasmtime::Result<BotExecutionData>> {
        self.new_bot_sender.lock().unwrap().clone()
    }

    pub fn handle_new_bots(
        &mut self,
        commands: &mut Commands,
        track: &Track,
        bot_assets: &BotAssets,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
    ) {
        while let Ok(bot) = self.new_bot_receiver.lock().unwrap().try_recv() {
            match bot {
                Ok(bot) => {
                    self.bot_count += 1;
                    println!(
                        "new bot (number {}, steps {})",
                        self.bot_count,
                        bot.data.body_data.steps.len()
                    );
                    spawn_bot_visualization(
                        commands,
                        track,
                        bot.data,
                        bot.config,
                        self.bot_count,
                        bot_assets,
                        meshes,
                        materials,
                    );
                }
                Err(err) => {
                    self.error_message = Some(err.to_string());
                }
            }
        }
    }
}

fn runner_gui_update(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut gui_state: ResMut<RunnerGuiState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut camera: Query<(&mut PanOrbitCamera, &Transform)>,
    mut bot_vis: Query<(Entity, &mut BotVisualization)>,
    track: Res<Track>,
    time: Res<Time>,
    bot_assets: Res<BotAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let (mut po_camera, po_transform) = camera.single_mut()?;

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
                if icon_button(ui, ICON_HELP, size).clicked()
                    || keyboard_input.just_pressed(KeyCode::Slash)
                    || keyboard_input.just_pressed(KeyCode::F1)
                {
                    gui_state.help_open = true;
                }
                ui.separator();

                if icon_button(ui, ICON_SKIP_PREVIOUS, size).clicked()
                    || keyboard_input.just_pressed(KeyCode::Home)
                {
                    gui_state.play_time_sec = 0.0;
                }

                let rew_button = icon_button(ui, ICON_FAST_REWIND, size).clicked();

                if gui_state.play_active {
                    if icon_button(ui, ICON_PAUSE, size).clicked()
                        || keyboard_input.just_pressed(KeyCode::Space)
                    {
                        gui_state.play_active = false;
                    }
                } else {
                    if icon_button(ui, ICON_PLAY_ARROW, size).clicked()
                        || keyboard_input.just_pressed(KeyCode::Space)
                    {
                        gui_state.play_active = true;
                    }
                }

                let fwd_button = icon_button(ui, ICON_FAST_FORWARD, size).clicked();

                if icon_button(ui, ICON_SKIP_NEXT, size).clicked()
                    || keyboard_input.just_pressed(KeyCode::End)
                {
                    gui_state.play_time_sec = gui_state.play_max_sec;
                }

                let shift = keyboard_input.pressed(KeyCode::ShiftLeft)
                    || keyboard_input.pressed(KeyCode::ShiftRight);
                let ctrl = keyboard_input.pressed(KeyCode::ControlLeft)
                    || keyboard_input.pressed(KeyCode::ControlRight);
                let rew_pressed = keyboard_input.pressed(KeyCode::Comma)
                    || keyboard_input.pressed(KeyCode::PageDown);
                let fwd_pressed = keyboard_input.pressed(KeyCode::Period)
                    || keyboard_input.pressed(KeyCode::PageUp);
                let rew_clicked = keyboard_input.just_pressed(KeyCode::Comma)
                    || keyboard_input.just_pressed(KeyCode::PageDown);
                let fwd_clicked = keyboard_input.just_pressed(KeyCode::Period)
                    || keyboard_input.just_pressed(KeyCode::PageUp);

                if shift && ctrl {
                    if rew_clicked {
                        gui_state.play_time_sec -= 0.001;
                    }
                    if fwd_clicked {
                        gui_state.play_time_sec += 0.001;
                    }
                } else if shift || ctrl {
                    if rew_pressed || rew_button {
                        gui_state.play_time_sec -= 0.001;
                    }
                    if fwd_pressed || fwd_button {
                        gui_state.play_time_sec += 0.001;
                    }
                } else {
                    if rew_clicked || rew_button {
                        gui_state.play_time_sec -= 1.0;
                        gui_state.play_time_sec = gui_state.play_time_sec.round();
                    }
                    if fwd_clicked || fwd_button {
                        gui_state.play_time_sec += 1.0;
                        gui_state.play_time_sec = gui_state.play_time_sec.round();
                    }
                }

                // Clamp time again (it could have been changed by user commands)
                gui_state.play_time_sec =
                    gui_state.play_time_sec.min(gui_state.play_max_sec).max(0.0);

                rl(ui, format!("{:.3}", gui_state.play_time_sec), size);

                ui.add_space(size / 2.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icon_button(ui, ICON_EXIT_TO_APP, size).clicked()
                        || keyboard_input.just_released(KeyCode::KeyQ)
                    {
                        exit.write(AppExit::Success);
                    }
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
                });

                gui_state.file_dialog.update(ctx);
                if let Some(path) = gui_state.file_dialog.take_picked() {
                    let sender = gui_state.get_bot_sender();
                    let output = gui_state.output.clone();
                    let logs = gui_state.logs;
                    let period = gui_state.period;
                    let start_time = gui_state.start_time;
                    let track = track.clone();
                    std::thread::spawn(move || {
                        process_new_bot(path, output, logs, track, period, start_time, sender);
                    });
                }

                gui_state.handle_new_bots(
                    &mut commands,
                    &track,
                    &bot_assets,
                    &mut meshes,
                    &mut materials,
                );
            });

            let base_text_size = gui_state.base_text_size;
            error_dialog(ui, &mut gui_state.error_message, base_text_size);
            help_dialog(ui, &mut gui_state.help_open, base_text_size);
        });

    let cb_size = gui_state.base_text_size * 3.0;
    egui::SidePanel::left("left_panel")
        .resizable(false)
        .default_width(cb_size * 3.0)
        .show_separator_line(false)
        .show(ctx, |ui| {
            camera_buttons(ui, gui_state.base_text_size, po_camera.as_mut());
        });

    egui::SidePanel::right("right_panel")
        .resizable(false)
        .default_width(gui_state.base_text_size * 30.0)
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                rl(ui, "Robots", gui_state.base_text_size);
                ui.separator();

                let mut bots = bot_vis.iter_mut().collect::<Vec<_>>();
                bots.sort_by_key(|(_bot_id, bot)| bot.bot_final_status);
                for (index, (_, bot)) in bots.iter_mut().enumerate() {
                    bot.bot_number = index;
                }
                bots.reverse();

                for (bot_id, bot) in bots.iter() {
                    ui.horizontal(|ui| {
                        if bot_status(
                            ui,
                            &bot.config.name,
                            Color32::from_rgb(
                                bot.config.color_main.r,
                                bot.config.color_main.g,
                                bot.config.color_main.b,
                            ),
                            Color32::from_rgb(
                                bot.config.color_secondary.r,
                                bot.config.color_secondary.g,
                                bot.config.color_secondary.b,
                            ),
                            bot.bot_activity.status_at_time(gui_state.play_time_sec),
                            gui_state.base_text_size,
                        ) {
                            gui_state.as_mut().bot_with_pending_remove = Some((
                                *bot_id,
                                BotName {
                                    name: bot.config.name.clone(),
                                    c1: Color32::from_rgb(
                                        bot.config.color_main.r,
                                        bot.config.color_main.g,
                                        bot.config.color_main.b,
                                    ),
                                    c2: Color32::from_rgb(
                                        bot.config.color_secondary.r,
                                        bot.config.color_secondary.g,
                                        bot.config.color_secondary.b,
                                    ),
                                },
                            ))
                        }
                    });
                }
            });

            if let Some(bot_id) = ask_bot_remove(ui, gui_state.as_mut()) {
                commands.entity(bot_id).despawn();
                gui_state.as_mut().bot_with_pending_remove = None;
            }
        });

    keyboard_camera_control(
        &mut po_camera,
        &po_transform,
        &keyboard_input,
        time.delta_secs(),
    );

    Ok(())
}

fn ask_bot_remove(ui: &mut Ui, gui_state: &mut RunnerGuiState) -> Option<Entity> {
    let mut response = None;
    if let Some((bot_id, bot_with_pending_remove)) = &gui_state.bot_with_pending_remove {
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
                    response = Some(*bot_id);
                }
                if no {
                    response = None;
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

trait BotstatusExt {
    fn color(&self) -> Color32;
    fn timer(&self) -> f32;
}

impl BotstatusExt for BotStatus {
    fn color(&self) -> Color32 {
        match self {
            BotStatus::Waiting { .. } => Color32::YELLOW,
            BotStatus::Racing { .. } => Color32::GREEN,
            BotStatus::EndedAt { .. } => Color32::GREEN,
            BotStatus::OutAt { .. } => Color32::RED,
        }
    }

    fn timer(&self) -> f32 {
        self.display_time_secs()
    }
}

fn bot_status(
    ui: &mut Ui,
    name: &str,
    c1: Color32,
    c2: Color32,
    status: BotStatus,
    base_text_size: f32,
) -> bool {
    let mut response = false;
    ui.horizontal(|ui| {
        response = bot_name(ui, name, c1, c2, base_text_size);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{:.2}", status.timer()))
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
