use std::f32::consts::{FRAC_PI_2, FRAC_PI_3};

use bevy::ecs::system::Commands;
use bevy::prelude::*;
use execution_data::{BodyExecutionData, WheelExecutionData};
use executor::wasm_bindings::exports::robot::Configuration;

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

pub fn spawn_bot_body(
    commands: &mut Commands,
    parent: Entity,
    configuration: &Configuration,
    assets: &BotAssets,
    materials: &mut Assets<StandardMaterial>,
    data: Option<BodyExecutionData>,
) -> Entity {
    let id = commands.spawn((ChildOf(parent), Transform::default())).id();
    if let Some(data) = data {
        commands.entity(id).insert(data);
    }

    let (color_main_material, color_secondary_material) =
        configuration.setup_color_materials(materials);

    let wheel_diameter = configuration.wheel_diameter / 1000.0;

    const BODY_THICKNESS: f32 = 0.004;
    const BODY_TO_WHEEL: f32 = 0.005;

    const SENSOR_LINK_D: f32 = BODY_THICKNESS * 0.8;
    const SENSOR_LENGHT: f32 = BODY_THICKNESS * 3.0;

    const SENSOR_CHIP_D: f32 = 0.001;

    const BACK_BUMPER_D: f32 = SENSOR_LINK_D;
    const FRONT_SUPPORT_D: f32 = SENSOR_LINK_D;

    let body_width = configuration.width_axle / 1000.0 - 2.0 * BODY_TO_WHEEL;

    let body_top = wheel_diameter.max(configuration.clearing_back / 1000.0 + BODY_THICKNESS);

    let body_motors_d = wheel_diameter * 0.8;
    let body_motors_h = body_top - wheel_diameter / 2.0;
    let body_back_h = body_top - configuration.clearing_back / 1000.0;

    let body_back_width = body_width * 0.8;

    let sensors_width = (configuration.front_sensors_spacing / 1000.0) * 16.0;
    let sensor_link_x = (body_width * 0.3).min(configuration.front_sensors_spacing / 1000.0 * 7.0);

    let sensors_height = configuration.front_sensors_height / 1000.0;
    let sensors_thickness =
        BODY_THICKNESS.max((wheel_diameter - body_motors_d) / 2.0 - sensors_height + SENSOR_LINK_D);

    let sensors_z = sensors_height + (sensors_thickness - wheel_diameter) / 2.0;

    let support_ground_z = (FRONT_SUPPORT_D - wheel_diameter) / 2.0;
    let support_height = sensors_height + sensors_thickness - FRONT_SUPPORT_D / 2.0;

    // axle
    let axle_d = 0.003;
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(assets.materials.black.clone()),
        Transform::from_scale(Vec3::new(axle_d, configuration.width_axle / 1000.0, axle_d))
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    ));

    // motor cylinder
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(color_main_material.clone()),
        Transform::from_scale(Vec3::new(body_motors_d, body_width, body_motors_d))
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    ));

    // body motors
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cube.clone()),
        MeshMaterial3d(color_main_material.clone()),
        Transform::from_scale(Vec3::new(body_width, body_motors_d, body_motors_h))
            .with_translation(Vec3::new(0.0, 0.0, body_motors_h / 2.0)),
    ));

    // body back
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cube.clone()),
        MeshMaterial3d(color_main_material.clone()),
        Transform::from_scale(Vec3::new(
            body_back_width,
            configuration.length_back / 1000.0,
            body_back_h,
        ))
        .with_translation(Vec3::new(
            0.0,
            -configuration.length_back / 2000.0,
            body_top - (wheel_diameter + body_back_h) / 2.0,
        )),
    ));

    // body back bumper
    let back_bumper_z =
        configuration.clearing_back / 1000.0 + (BACK_BUMPER_D - wheel_diameter) / 2.0;
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.sphere.clone()),
        MeshMaterial3d(assets.materials.black.clone()),
        Transform::from_scale(Vec3::ONE * BACK_BUMPER_D).with_translation(Vec3::new(
            -body_back_width / 2.0,
            -configuration.length_back / 1000.0,
            back_bumper_z,
        )),
    ));
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.sphere.clone()),
        MeshMaterial3d(assets.materials.black.clone()),
        Transform::from_scale(Vec3::ONE * BACK_BUMPER_D).with_translation(Vec3::new(
            body_back_width / 2.0,
            -configuration.length_back / 1000.0,
            back_bumper_z,
        )),
    ));
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(assets.materials.black.clone()),
        Transform::from_scale(Vec3::new(BACK_BUMPER_D, body_back_width, BACK_BUMPER_D))
            .with_translation(Vec3::new(
                0.0,
                -configuration.length_back / 1000.0,
                back_bumper_z,
            ))
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    ));

    // sensor plate
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cube.clone()),
        MeshMaterial3d(color_main_material.clone()),
        Transform::from_scale(Vec3::new(sensors_width, SENSOR_LENGHT, sensors_thickness))
            .with_translation(Vec3::new(
                0.0,
                configuration.length_front / 1000.0,
                sensors_z,
            )),
    ));

    // sensor chips
    for i in (0..16).into_iter().map(|i| i as f32 - 7.5) {
        commands.spawn((
            ChildOf(id),
            Mesh3d(assets.meshes.sphere.clone()),
            MeshMaterial3d(assets.materials.black.clone()),
            Transform::from_scale(Vec3::ONE * SENSOR_CHIP_D).with_translation(Vec3::new(
                i * configuration.front_sensors_spacing / 1000.0,
                configuration.length_front / 1000.0,
                sensors_z - sensors_thickness / 2.0,
            )),
        ));
        commands.spawn((
            ChildOf(id),
            Mesh3d(assets.meshes.sphere.clone()),
            MeshMaterial3d(assets.materials.black.clone()),
            Transform::from_scale(Vec3::ONE * SENSOR_CHIP_D).with_translation(Vec3::new(
                i * configuration.front_sensors_spacing / 1000.0,
                configuration.length_front / 1000.0,
                sensors_z + sensors_thickness / 2.0,
            )),
        ));
    }

    // sensor link
    for i in [-1.0, 1.0] {
        commands.spawn((
            ChildOf(id),
            Mesh3d(assets.meshes.cylinder.clone()),
            MeshMaterial3d(color_secondary_material.clone()),
            Transform::from_scale(Vec3::new(
                SENSOR_LINK_D,
                configuration.length_front / 1000.0,
                SENSOR_LINK_D,
            ))
            .with_translation(Vec3::new(
                i * sensor_link_x,
                configuration.length_front / 2000.0,
                sensors_z,
            )),
        ));
    }

    // front support
    for i in [-1.0, 1.0] {
        commands.spawn((
            ChildOf(id),
            Mesh3d(assets.meshes.sphere.clone()),
            MeshMaterial3d(assets.materials.black.clone()),
            Transform::from_scale(Vec3::ONE * FRONT_SUPPORT_D).with_translation(Vec3::new(
                i * sensors_width / 2.0,
                configuration.length_front / 1000.0,
                support_ground_z,
            )),
        ));
        commands.spawn((
            ChildOf(id),
            Mesh3d(assets.meshes.sphere.clone()),
            MeshMaterial3d(assets.materials.black.clone()),
            Transform::from_scale(Vec3::ONE * FRONT_SUPPORT_D).with_translation(Vec3::new(
                i * sensors_width / 2.0,
                configuration.length_front / 1000.0,
                sensors_z + sensors_thickness / 2.0,
            )),
        ));
        commands.spawn((
            ChildOf(id),
            Mesh3d(assets.meshes.cylinder.clone()),
            MeshMaterial3d(assets.materials.black.clone()),
            Transform::from_scale(Vec3::new(FRONT_SUPPORT_D, support_height, FRONT_SUPPORT_D))
                .with_translation(Vec3::new(
                    i * sensors_width / 2.0,
                    configuration.length_front / 1000.0,
                    (support_height + FRONT_SUPPORT_D - wheel_diameter) / 2.0,
                ))
                .with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
        ));
    }

    id
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
    let wheel_world = Vec3::new((configuration.width_axle / 2000.0) * -side.sign(), 0.0, 0.0);

    let (_, color_secondary_material) = configuration.setup_color_materials(materials);

    let transform = data
        .as_ref()
        .map(|data| Transform::from_translation(wheel_world))
        .unwrap_or_default();

    let id = commands.spawn((ChildOf(parent), transform)).id();

    if let Some(data) = data {
        commands.entity(id).insert(data);
    }

    let wheel_d = configuration.wheel_diameter / 1000.0;
    let wheel_w = 0.02; // wheel_d * 3.0 / 2.0;

    // cylinder mesh
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(color_secondary_material.clone()),
        Transform::from_translation(Vec3::X * -side.sign() * wheel_w / 2.0)
            .with_scale(Vec3::new(wheel_d, wheel_w, wheel_d))
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    ));

    // ext drawing
    let drawing_out = 0.001;
    let ext_cyl_tranform = Vec3::new(wheel_d / 3.5, drawing_out / 2.0, wheel_d / 2.0);
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(assets.materials.black.clone()),
        Transform::from_translation(Vec3::new(-side.sign() * wheel_w, wheel_d / 4.0, 0.0))
            .with_scale(ext_cyl_tranform)
            .with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
    ));
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(assets.materials.black.clone()),
        Transform::from_translation(Vec3::new(
            -side.sign() * wheel_w,
            -(wheel_d / 4.0) * FRAC_PI_3.cos(),
            -(wheel_d / 4.0) * FRAC_PI_3.sin(),
        ))
        .with_scale(ext_cyl_tranform)
        .with_rotation(Quat::from_euler(EulerRot::XYZ, FRAC_PI_3, 0.0, FRAC_PI_2)),
    ));
    commands.spawn((
        ChildOf(id),
        Mesh3d(assets.meshes.cylinder.clone()),
        MeshMaterial3d(assets.materials.black.clone()),
        Transform::from_translation(Vec3::new(
            -side.sign() * wheel_w,
            -(wheel_d / 4.0) * FRAC_PI_3.cos(),
            (wheel_d / 4.0) * FRAC_PI_3.sin(),
        ))
        .with_scale(ext_cyl_tranform)
        .with_rotation(Quat::from_euler(EulerRot::XYZ, -FRAC_PI_3, 0.0, FRAC_PI_2)),
    ));
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
