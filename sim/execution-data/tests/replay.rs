//! Tests for the replay model: a recorded run is an array, and playback is an
//! index into it. That is what makes scrubbing, single-tick stepping and
//! backwards playback free.

use bevy::math::Vec3;
use bevy::transform::components::Transform;
use execution_data::{
    ActivityData, BodyExecutionData, BotFinalStatus, BotStatus, WheelDataSide,
    WheelExecutionData,
};

const PERIOD_US: u32 = 500;

fn body_data(count: usize) -> BodyExecutionData {
    let mut data = BodyExecutionData::empty(PERIOD_US);
    for i in 0..count {
        data.steps.push(Transform::from_xyz(i as f32, 0.0, 0.0));
    }
    data
}

// --- indexing ----------------------------------------------------------------

#[test]
fn playback_indexes_the_recording_by_period() {
    let data = body_data(10);

    assert_eq!(data.at_time_secs(0.0).translation.x, 0.0);
    assert_eq!(data.at_time_secs(0.0005).translation.x, 1.0, "one 500us tick");
    assert_eq!(data.at_time_secs(0.0010).translation.x, 2.0);
    // Mid-tick truncates to the sample that was current at that instant.
    assert_eq!(data.at_time_secs(0.00075).translation.x, 1.0);
}

#[test]
fn playback_clamps_outside_the_recording() {
    let data = body_data(10);

    assert_eq!(data.at_time_secs(-5.0).translation.x, 0.0, "before the start");
    assert_eq!(data.at_time_secs(999.0).translation.x, 9.0, "after the end");
}

#[test]
fn an_empty_recording_is_not_a_panic() {
    // A robot that traps during `setup` produces exactly this.
    let data = BodyExecutionData::empty(PERIOD_US);
    assert_eq!(data.at_time_secs(1.0).translation, Vec3::ZERO);

    let wheel = WheelExecutionData::empty(PERIOD_US, WheelDataSide::Left);
    assert_eq!(wheel.at_time_secs(1.0), 0.0);
}

#[test]
fn wheel_playback_indexes_the_same_way() {
    let mut wheel = WheelExecutionData::empty(PERIOD_US, WheelDataSide::Right);
    wheel.steps.extend([0.0, 0.5, 1.0, 1.5]);

    assert_eq!(wheel.at_time_secs(0.0), 0.0);
    assert_eq!(wheel.at_time_secs(0.0010), 1.0);
    assert_eq!(wheel.at_time_secs(99.0), 1.5);
}

// --- activity ----------------------------------------------------------------

/// A robot that started at 1 s and finished at 2 s.
fn finisher() -> ActivityData {
    ActivityData {
        start_time_us: Some(1_000_000),
        out_time_us: None,
        end_time_us: Some(2_000_000),
    }
}

#[test]
fn status_is_reported_relative_to_the_start_signal() {
    let a = finisher();

    assert!(matches!(a.status_at_time(0.5), BotStatus::Waiting { .. }));

    // Half a second into the race, not 1.5 s: the countdown is not charged to
    // anyone's lap time.
    match a.status_at_time(1.5) {
        BotStatus::Racing { time_secs } => assert!((time_secs - 0.5).abs() < 1e-5),
        _ => panic!("should be racing"),
    }

    match a.status_at_time(2.5) {
        BotStatus::EndedAt { time_secs } => assert!((time_secs - 1.0).abs() < 1e-5),
        _ => panic!("should have ended"),
    }
}

#[test]
fn a_robot_is_active_only_between_start_and_finish() {
    assert!(!ActivityData::empty(false).is_active_now(), "not started");
    assert!(ActivityData::empty(true).is_active_now(), "forced started");
    assert!(!finisher().is_active_now(), "already finished");

    let racing = ActivityData {
        start_time_us: Some(1_000_000),
        out_time_us: None,
        end_time_us: None,
    };
    assert!(racing.is_active_now());
}

#[test]
fn final_status_summarises_the_run() {
    assert!(matches!(
        finisher().final_status(),
        BotFinalStatus::EndedAt { .. }
    ));
    assert!(matches!(
        ActivityData::empty(false).final_status(),
        BotFinalStatus::NotStarted
    ));
    assert!(matches!(
        ActivityData::empty(true).final_status(),
        BotFinalStatus::NotEnded
    ));

    let crashed = ActivityData {
        start_time_us: Some(1_000_000),
        out_time_us: Some(1_750_000),
        end_time_us: None,
    };
    match crashed.final_status() {
        BotFinalStatus::OutAt { time_secs } => assert!((time_secs - 0.75).abs() < 1e-5),
        _ => panic!("should be out"),
    }
}

// --- ranking -----------------------------------------------------------------

#[test]
fn ranking_orders_by_category_then_by_time() {
    // Ascending order is best-first: finishers beat crashers, crashers beat
    // robots that never finished, and the fastest finisher wins.
    let fast = BotFinalStatus::EndedAt { time_secs: 10.0 };
    let slow = BotFinalStatus::EndedAt { time_secs: 20.0 };
    let early_crash = BotFinalStatus::OutAt { time_secs: 2.0 };
    let late_crash = BotFinalStatus::OutAt { time_secs: 30.0 };

    assert!(fast < slow, "faster finisher ranks better");
    assert!(slow < early_crash, "any finisher beats any crasher");
    assert!(early_crash < late_crash, "crash time still orders crashers");
    assert!(late_crash < BotFinalStatus::NotEnded);
    assert!(BotFinalStatus::NotEnded < BotFinalStatus::NotStarted);
}

#[test]
fn sorting_a_field_puts_the_winner_first() {
    let mut field = vec![
        BotFinalStatus::NotStarted,
        BotFinalStatus::OutAt { time_secs: 5.0 },
        BotFinalStatus::EndedAt { time_secs: 12.5 },
        BotFinalStatus::NotEnded,
        BotFinalStatus::EndedAt { time_secs: 11.0 },
    ];
    field.sort();

    assert!(matches!(field[0], BotFinalStatus::EndedAt { time_secs } if time_secs == 11.0));
    assert!(matches!(field[1], BotFinalStatus::EndedAt { time_secs } if time_secs == 12.5));
    assert!(matches!(field[2], BotFinalStatus::OutAt { .. }));
    assert!(matches!(field[3], BotFinalStatus::NotEnded));
    assert!(matches!(field[4], BotFinalStatus::NotStarted));
}
