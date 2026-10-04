use std::f32::consts::{FRAC_PI_2, FRAC_PI_3};

use bevy::ecs::system::Commands;
use bevy::prelude::*;
use execution_data::{BodyExecutionData, WheelExecutionData};
use executor::wasm_host::exports::robot::Configuration;

use crate::utils::Side;

use super::motors::Wheel;
use super::{BotBodyMarker, BotConfigurationResource};

pub struct BotMeshes {
    pub cube: Handle<Mesh>,
    pub cylinder: Handle<Mesh>,
    pub sphere: Handle<Mesh>,
}

pub struct BotMaterials {
    pub black: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct BotAssets {
    pub meshes: BotMeshes,
    pub materials: BotMaterials,
}

pub fn setup_bot_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = meshes.add(Cuboid::from_size(Vec3::ONE));
    let cylinder_mesh = meshes.add(Cylinder::new(0.5, 1.0));
    let sphere_mesh = meshes.add(Sphere::new(0.5));

    let black_material = materials.add(Color::srgb(0.0, 0.0, 0.0));

    let assets = BotAssets {
        meshes: BotMeshes {
            cube: cube_mesh.clone(),
            cylinder: cylinder_mesh.clone(),
            sphere: sphere_mesh.clone(),
        },
        materials: BotMaterials {
            black: black_material.clone(),
        },
    };

    commands.insert_resource(assets);
}

trait SetupColorMaterials {
    fn setup_color_materials(
        &self,
        materials: &mut Assets<StandardMaterial>,
    ) -> (Handle<StandardMaterial>, Handle<StandardMaterial>);
}

impl SetupColorMaterials for Configuration {
    fn setup_color_materials(
        &self,
        materials: &mut Assets<StandardMaterial>,
    ) -> (Handle<StandardMaterial>, Handle<StandardMaterial>) {
        let color_main = Color::srgb(
            self.color_main.r as f32 / u8::max_value() as f32,
            self.color_main.g as f32 / u8::max_value() as f32,
            self.color_main.b as f32 / u8::max_value() as f32,
        );
        let color_secondary = Color::srgb(
            self.color_secondary.r as f32 / u8::max_value() as f32,
            self.color_secondary.g as f32 / u8::max_value() as f32,
            self.color_secondary.b as f32 / u8::max_value() as f32,
        );

        (materials.add(color_main), materials.add(color_secondary))
    }
}

/// A single visual part of the bot: a primitive mesh with a material and a transform.
fn part(
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    transform: Transform,
) -> impl Scene + use<> {
    let (mesh, material) = (mesh.clone(), material.clone());
    bsn! {
        Mesh3d(mesh)
        MeshMaterial3d::<StandardMaterial>(material)
        template_value(transform)
    }
}

pub fn spawn_bot_body(
    commands: &mut Commands,
    parent: Entity,
    configuration: &Configuration,
    assets: &BotAssets,
    materials: &mut Assets<StandardMaterial>,
    data: Option<BodyExecutionData>,
) -> Entity {
    let (color_main_material, color_secondary_material) =
        configuration.setup_color_materials(materials);
    let BotMeshes {
        cube,
        cylinder,
        sphere,
    } = &assets.meshes;
    let black = &assets.materials.black;
    let main = &color_main_material;
    let secondary = &color_secondary_material;

    let wheel_diameter = configuration.wheel_diameter / 1000.0;
    let width_axle = configuration.width_axle / 1000.0;
    let length_back = configuration.length_back / 1000.0;
    let length_front = configuration.length_front / 1000.0;
    let sensors_spacing = configuration.front_sensors_spacing / 1000.0;
    let clearing_back = configuration.clearing_back / 1000.0;

    const BODY_THICKNESS: f32 = 0.004;
    const BODY_TO_WHEEL: f32 = 0.005;

    const SENSOR_LINK_D: f32 = BODY_THICKNESS * 0.8;
    const SENSOR_LENGHT: f32 = BODY_THICKNESS * 3.0;

    const SENSOR_CHIP_D: f32 = 0.001;

    const BACK_BUMPER_D: f32 = SENSOR_LINK_D;
    const FRONT_SUPPORT_D: f32 = SENSOR_LINK_D;

    const AXLE_D: f32 = 0.003;

    let body_width = width_axle - 2.0 * BODY_TO_WHEEL;

    let body_top = wheel_diameter.max(clearing_back + BODY_THICKNESS);

    let body_motors_d = wheel_diameter * 0.8;
    let body_motors_h = body_top - wheel_diameter / 2.0;
    let body_back_h = body_top - clearing_back;

    let body_back_width = body_width * 0.8;

    let sensors_width = sensors_spacing * 16.0;
    let sensor_link_x = (body_width * 0.3).min(sensors_spacing * 7.0);

    let sensors_height = configuration.front_sensors_height / 1000.0;
    let sensors_thickness =
        BODY_THICKNESS.max((wheel_diameter - body_motors_d) / 2.0 - sensors_height + SENSOR_LINK_D);

    let sensors_z = sensors_height + (sensors_thickness - wheel_diameter) / 2.0;

    let support_ground_z = (FRONT_SUPPORT_D - wheel_diameter) / 2.0;
    let support_height = sensors_height + sensors_thickness - FRONT_SUPPORT_D / 2.0;

    let back_bumper_z = clearing_back + (BACK_BUMPER_D - wheel_diameter) / 2.0;

    let rot_z = Quat::from_rotation_z(FRAC_PI_2);
    let sphere_at = |d: f32, x: f32, y: f32, z: f32| {
        part(
            sphere,
            black,
            Transform::from_xyz(x, y, z).with_scale(Vec3::splat(d)),
        )
    };

    let sensor_chips: Vec<_> = (0..16)
        .map(|i| (i as f32 - 7.5) * sensors_spacing)
        .flat_map(|x| {
            [
                sensors_z - sensors_thickness / 2.0,
                sensors_z + sensors_thickness / 2.0,
            ]
            .map(|z| sphere_at(SENSOR_CHIP_D, x, length_front, z))
        })
        .collect();

    let sensor_links: Vec<_> = [-1.0, 1.0]
        .map(|side| {
            part(
                cylinder,
                secondary,
                Transform::from_xyz(side * sensor_link_x, length_front / 2.0, sensors_z)
                    .with_scale(Vec3::new(SENSOR_LINK_D, length_front, SENSOR_LINK_D)),
            )
        })
        .into();

    let front_supports: Vec<Box<dyn Scene>> = [-1.0, 1.0]
        .into_iter()
        .flat_map(|side| {
            let x = side * sensors_width / 2.0;
            [
                Box::new(sphere_at(
                    FRONT_SUPPORT_D,
                    x,
                    length_front,
                    support_ground_z,
                )) as Box<dyn Scene>,
                Box::new(sphere_at(
                    FRONT_SUPPORT_D,
                    x,
                    length_front,
                    sensors_z + sensors_thickness / 2.0,
                )),
                Box::new(part(
                    cylinder,
                    black,
                    Transform::from_xyz(
                        x,
                        length_front,
                        (support_height + FRONT_SUPPORT_D - wheel_diameter) / 2.0,
                    )
                    .with_scale(Vec3::new(FRONT_SUPPORT_D, support_height, FRONT_SUPPORT_D))
                    .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
                )),
            ]
        })
        .collect();

    let mut body = commands.spawn_scene(bsn! {
        ChildOf(parent)
        Transform
        Children [
            // axle
            part(cylinder, black, {
                Transform::from_scale(Vec3::new(AXLE_D, width_axle, AXLE_D)).with_rotation(rot_z)
            }),
            // motor cylinder
            part(cylinder, main, {
                Transform::from_scale(Vec3::new(body_motors_d, body_width, body_motors_d))
                    .with_rotation(rot_z)
            }),
            // body motors
            part(cube, main, {
                Transform::from_xyz(0.0, 0.0, body_motors_h / 2.0)
                    .with_scale(Vec3::new(body_width, body_motors_d, body_motors_h))
            }),
            // body back
            part(cube, main, {
                Transform::from_xyz(0.0, -length_back / 2.0, body_top - (wheel_diameter + body_back_h) / 2.0)
                    .with_scale(Vec3::new(body_back_width, length_back, body_back_h))
            }),
            // body back bumper
            sphere_at(BACK_BUMPER_D, -body_back_width / 2.0, -length_back, back_bumper_z),
            sphere_at(BACK_BUMPER_D, body_back_width / 2.0, -length_back, back_bumper_z),
            part(cylinder, black, {
                Transform::from_xyz(0.0, -length_back, back_bumper_z)
                    .with_scale(Vec3::new(BACK_BUMPER_D, body_back_width, BACK_BUMPER_D))
                    .with_rotation(rot_z)
            }),
            // sensor plate
            part(cube, main, {
                Transform::from_xyz(0.0, length_front, sensors_z)
                    .with_scale(Vec3::new(sensors_width, SENSOR_LENGHT, sensors_thickness))
            }),
            {sensor_chips},
            {sensor_links},
            {front_supports},
        ]
    });
    if let Some(data) = data {
        body.insert(data);
    }
    body.id()
}

pub fn spawn_bot_wheel(
    commands: &mut Commands,
    parent: Entity,
    configuration: &Configuration,
    assets: &BotAssets,
    materials: &mut Assets<StandardMaterial>,
    side: Side,
    data: Option<WheelExecutionData>,
) {
    let (_, color_secondary_material) = configuration.setup_color_materials(materials);
    let cylinder = &assets.meshes.cylinder;
    let black = &assets.materials.black;

    let wheel_world = Vec3::new((configuration.width_axle / 2000.0) * -side.sign(), 0.0, 0.0);
    let transform = if data.is_some() {
        Transform::from_translation(wheel_world)
    } else {
        Transform::default()
    };

    let wheel_d = configuration.wheel_diameter / 1000.0;
    let wheel_w = 0.02; // wheel_d * 3.0 / 2.0;

    // ext drawing
    let drawing_out = 0.001;
    let ext_x = -side.sign() * wheel_w;
    let ext_scale = Vec3::new(wheel_d / 3.5, drawing_out / 2.0, wheel_d / 2.0);
    let ext_cos = (wheel_d / 4.0) * FRAC_PI_3.cos();
    let ext_sin = (wheel_d / 4.0) * FRAC_PI_3.sin();
    let ext = |y: f32, z: f32, angle_x: f32| {
        part(
            cylinder,
            black,
            Transform::from_xyz(ext_x, y, z)
                .with_scale(ext_scale)
                .with_rotation(Quat::from_euler(EulerRot::XYZ, angle_x, 0.0, FRAC_PI_2)),
        )
    };

    let mut wheel = commands.spawn_scene(bsn! {
        ChildOf(parent)
        template_value(transform)
        Children [
            // cylinder mesh
            part(cylinder, &color_secondary_material, {
                Transform::from_translation(Vec3::X * -side.sign() * wheel_w / 2.0)
                    .with_scale(Vec3::new(wheel_d, wheel_w, wheel_d))
                    .with_rotation(Quat::from_rotation_z(FRAC_PI_2))
            }),
            ext(wheel_d / 4.0, 0.0, 0.0),
            ext(-ext_cos, -ext_sin, FRAC_PI_3),
            ext(-ext_cos, ext_sin, -FRAC_PI_3),
        ]
    });
    if let Some(data) = data {
        wheel.insert(data);
    }
}

pub fn setup_test_bot_visualizer(
    mut commands: Commands,
    assets: Res<BotAssets>,
    configuration: Res<BotConfigurationResource>,
    body_query: Query<Entity, With<BotBodyMarker>>,
    wheels_query: Query<(Entity, &Wheel)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cfg = configuration.cfg();

    let body = body_query.single().unwrap();
    spawn_bot_body(&mut commands, body, &cfg, &assets, &mut materials, None);

    for (wheel_id, wheel) in wheels_query.iter() {
        spawn_bot_wheel(
            &mut commands,
            wheel_id,
            &cfg,
            &assets,
            &mut materials,
            wheel.side,
            None,
        );
    }
}
