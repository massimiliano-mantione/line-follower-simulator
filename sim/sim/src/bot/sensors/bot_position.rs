use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::track::TrackSegment;
use execution_data::{BotPhysicalPosition, BotPosition, SensorsData};

#[derive(Component, Default, Clone)]
pub struct BotPositionDetector {}

pub fn compute_bot_position(
    read_rapier_context: ReadRapierContext,
    bot_query: Query<&Transform, With<BotPositionDetector>>,
    track_segments_query: Query<&TrackSegment>,
    mut sensors_data: ResMut<SensorsData>,
) {
    let rapier_context = read_rapier_context.single().unwrap();
    let bot_transform = bot_query.single().unwrap();
    let origin = bot_transform.translation;
    let (bot_rx, bot_ry, bot_rz) = bot_transform.rotation.to_euler(EulerRot::XYZ);
    let dir = Vec3::NEG_Z;
    let max_toi = 0.1;

    sensors_data.bot_position = if let Some((entity, _)) = rapier_context.cast_ray_and_get_normal(
        origin,
        dir,
        max_toi,
        true,
        QueryFilter::default().predicate(&|entity| track_segments_query.get(entity).is_ok()),
    ) {
        // Bot is over the track
        if track_segments_query.get(entity).unwrap() == &TrackSegment::End {
            BotPosition::End
        } else {
            BotPosition::OnTrack
        }
    } else {
        // Bot is out
        BotPosition::Out
    };

    sensors_data.bot_physical_position = BotPhysicalPosition {
        pos: origin,
        rot: Vec3::new(bot_rx, bot_ry, bot_rz),
    };
}
