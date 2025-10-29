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
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use bevy_panorbit_camera::PanOrbitCamera;
use egui_material_icons::icons::{ICON_EXIT_TO_APP, ICON_HELP, ICON_ZOOM_IN, ICON_ZOOM_OUT};
use execution_data::{MotorDriversDutyCycles, PWM_MAX, SensorsData};

use crate::ui::{
    camera_buttons, error_dialog, help_dialog, icon_button, keyboard_camera_control, rl, rlc,
};

pub fn test_gui_setup(app: &mut App) {
    app.add_systems(EguiPrimaryContextPass, test_gui_update)
        .insert_resource(TestGuiState::default());
}

#[derive(Resource)]
struct TestGuiState {
    base_text_size: f32,
    pwm_fwd_cmd: i16,
    pwm_side_cmd: i16,
    error_message: Option<String>,
    help_open: bool,
}

impl Default for TestGuiState {
    fn default() -> Self {
        Self {
            base_text_size: 8.0,
            pwm_fwd_cmd: PWM_MAX / 9,
            pwm_side_cmd: PWM_MAX / 10,
            error_message: None,
            help_open: false,
        }
    }
}

fn test_gui_update(
    mut contexts: EguiContexts,
    mut gui_state: ResMut<TestGuiState>,
    keyboard_input: ResMut<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut pwm: ResMut<MotorDriversDutyCycles>,
    sensors: Res<SensorsData>,
    time: Res<Time>,
    mut camera: Query<(&mut PanOrbitCamera, &Transform)>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let (mut po_camera, po_transform) = camera.single_mut()?;

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

                let sensors_color = if sensors.is_over_track_end {
                    egui::Color32::WHITE
                } else if sensors.is_out_of_track {
                    egui::Color32::RED
                } else {
                    egui::Color32::GREEN
                };
                for sensor_index in 0..16 {
                    let value = (sensors.line_sensors[sensor_index] * 255.0 / 100.0) as u8;
                    rlc(ui, &format!("{:3}", value), size * 0.5, sensors_color);
                    ui.add_space(size / 2.0);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if icon_button(ui, ICON_EXIT_TO_APP, size).clicked()
                        || keyboard_input.just_released(KeyCode::KeyQ)
                    {
                        exit.write(AppExit::Success);
                    }
                    if icon_button(ui, ICON_ZOOM_IN, size).clicked() {
                        gui_state.base_text_size += 1.0;
                        gui_state.base_text_size = gui_state.base_text_size.max(3.0);
                    }
                    if icon_button(ui, ICON_ZOOM_OUT, size).clicked() {
                        gui_state.base_text_size -= 1.0;
                    }
                });
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

    // Handle motor control if keyboard_camera_control returns false
    if !keyboard_camera_control(
        &mut po_camera,
        &po_transform,
        &keyboard_input,
        time.delta_secs(),
    ) {
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

        pwm.left = (forward * gui_state.pwm_fwd_cmd + side * gui_state.pwm_side_cmd)
            .clamp(-PWM_MAX, PWM_MAX);
        pwm.right = (forward * gui_state.pwm_fwd_cmd - side * gui_state.pwm_side_cmd)
            .clamp(-PWM_MAX, PWM_MAX);
    }

    Ok(())
}
