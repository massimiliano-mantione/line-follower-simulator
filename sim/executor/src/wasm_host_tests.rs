//! Unit tests for the simulated clock, the stepping engine and the device
//! futures registry.
//!
//! Every test here runs against [`MockStepper`] rather than the Bevy simulator.
//! That is the whole point of the [`SimulationStepper`] seam: the clock and the
//! futures machinery are testable with no physics, no rendering and no WASM, in
//! milliseconds.
//!
//! The device methods on [`BotHost`] take the consumed fuel as a parameter, so a
//! test can impersonate a guest simply by passing the fuel balance it would have
//! had. Holding the fuel *constant* across calls is a useful trick: it proves that
//! any time that passed came from the stepping logic rather than from the guest
//! burning instructions.

use super::*;
use crate::mock_stepper::MockStepper;

/// One second of simulated race time.
const TOTAL: TimeUs = 1_000_000;
/// Simulation step period used by every test below.
const PERIOD: u32 = 500;

fn host() -> BotHost<MockStepper> {
    BotHost::new(MockStepper::new(PERIOD), TOTAL, None, false)
}

/// The fuel balance a guest would have left once the clock reads `t`.
fn fuel_at(t: TimeUs) -> u64 {
    fuel_for_time_us(TOTAL - t)
}

// --- the clock (lesson 02) ---------------------------------------------------

#[test]
fn fuel_and_time_are_inverses() {
    // A 20 MHz CPU: one fuel unit is one instruction is 50 ns, so 1 us buys 20.
    assert_eq!(fuel_for_time_us(1), 20);
    assert_eq!(fuel_for_time_us(1_000), 20_000);

    for t in [0u32, 1, 7, 500, 1_000, 999_999] {
        assert_eq!(time_us_for_fuel(fuel_for_time_us(t)), t, "round trip for {}", t);
    }
}

#[test]
fn clock_starts_at_zero() {
    assert_eq!(host().current_time().unwrap(), 0);
}

#[test]
fn clock_is_derived_from_consumed_fuel() {
    let mut h = host();
    let v = h
        .device_operation_immediate(fuel_at(1_300), DeviceOperation::GetTime)
        .unwrap();
    assert_eq!(v.get_u32(0), 1_300);
}

#[test]
fn clock_is_monotonic_across_operations() {
    let mut h = host();
    let mut last = 0;
    for t in [0u32, 250, 500, 1_000, 1_750, 10_000] {
        let v = h
            .device_operation_immediate(fuel_at(t), DeviceOperation::GetTime)
            .unwrap();
        let now = v.get_u32(0);
        assert!(now >= last, "clock went backwards: {} then {}", last, now);
        last = now;
    }
}

#[test]
fn write_line_charges_one_hundred_us_per_character() {
    let mut h = host();
    let fuel = fuel_at(1_000);

    h.write_line(fuel, "hello".to_string()).unwrap(); // 5 chars -> 500 us

    // Same fuel balance: the guest executed no further instructions, yet the clock
    // moved. Serial output costs simulated time even though it costs no fuel.
    assert_eq!(h.current_time().unwrap(), 1_500);
}

#[test]
fn write_file_charges_ten_us_per_byte() {
    let mut h = host();
    let fuel = fuel_at(1_000);

    h.write_file(fuel, "t".to_string(), vec![0u8; 50], None)
        .unwrap(); // 50 bytes -> 500 us

    assert_eq!(h.current_time().unwrap(), 1_500);
}

#[test]
fn exhausted_fuel_is_an_error() {
    // The only failure mode a runaway robot has: an infinite loop is not a hang,
    // it is a race that ends.
    let mut h = host();
    assert!(
        h.device_operation_immediate(0, DeviceOperation::GetTime)
            .is_err()
    );
}

// --- the stepping engine (lesson 03) -----------------------------------------

#[test]
fn blocking_sleep_advances_clock_and_steps_physics() {
    let mut h = host();
    h.device_operation_blocking(fuel_at(0), DeviceOperation::SleepFor(1_000))
        .unwrap();

    assert_eq!(h.current_time().unwrap(), 1_000);
    assert_eq!(h.stepper.get_step_count(), 2, "two 500us ticks fit in 1000us");
}

#[test]
fn sensor_read_snaps_to_the_next_step_boundary() {
    let mut h = host();
    // Asking at 1300us: the device samples on 500us boundaries, so the value is
    // not available until 1500us. The robot pays for asking at an awkward moment.
    h.device_operation_blocking(fuel_at(1_300), DeviceOperation::ReadLineLeft)
        .unwrap();

    assert_eq!(h.current_time().unwrap(), 1_500);
}

#[test]
fn ready_condition_rounds_up_to_a_step_boundary() {
    let stepper = MockStepper::new(PERIOD);

    // Already on a boundary: the sample is available now.
    assert!(matches!(
        DeviceOperation::ReadLineLeft.ready_condition(1_500, &stepper),
        FutureReadyCondition::ReadyAt(1_500)
    ));
    // Off a boundary: wait for the next sample.
    assert!(matches!(
        DeviceOperation::ReadLineLeft.ready_condition(1_300, &stepper),
        FutureReadyCondition::ReadyAt(1_500)
    ));
    // Slow devices wait proportionally longer.
    assert_eq!(DeviceOperation::ReadGyro.ready_steps(), READY_STEPS_GYRO);
    assert_eq!(
        DeviceOperation::ReadImuFusedData.ready_steps(),
        READY_STEPS_IMU_FUSED
    );
}

#[test]
fn host_state_reads_are_always_ready() {
    let stepper = MockStepper::new(PERIOD);
    for op in [
        DeviceOperation::GetTime,
        DeviceOperation::GetPeriod,
        DeviceOperation::GetEnabled,
    ] {
        assert!(
            matches!(
                op.ready_condition(1_300, &stepper),
                FutureReadyCondition::ReadyAt(1_300)
            ),
            "host state reads do not wait for a device"
        );
    }
}

#[test]
fn waiting_on_the_enable_signal_is_not_a_deadline() {
    let stepper = MockStepper::new(PERIOD);
    // The start signal cannot be expressed as a time, so the type system forces
    // us to model it as a condition instead.
    assert!(matches!(
        DeviceOperation::WaitEnabled.ready_condition(1_300, &stepper),
        FutureReadyCondition::IsActive
    ));
    assert!(matches!(
        DeviceOperation::WaitDisabled.ready_condition(1_300, &stepper),
        FutureReadyCondition::IsInactive
    ));
}

#[test]
fn immediate_operations_do_not_step_the_simulation() {
    let mut h = host();
    h.device_operation_immediate(fuel_at(1_300), DeviceOperation::GetTime)
        .unwrap();
    h.device_operation_immediate(fuel_at(1_300), DeviceOperation::GetPeriod)
        .unwrap();

    assert_eq!(h.stepper.get_step_count(), 0, "host state reads are free");
    assert_eq!(h.current_time().unwrap(), 1_300);
}

#[test]
fn setting_motor_power_catches_the_world_up_first() {
    let mut h = host();
    h.set_motors_power(fuel_at(1_200), 500, -500).unwrap();

    // Two full ticks fit before 1200us, and they must run under the *old* duty
    // cycle: stepping after applying would let the robot's decisions act
    // retroactively.
    assert_eq!(h.stepper.get_step_count(), 2);
}

// --- device futures (lesson 07) ----------------------------------------------

#[test]
fn async_operation_becomes_ready_at_its_deadline() {
    let mut h = host();
    let fuel = fuel_at(0);

    h.poll_loop(fuel, true).unwrap();
    let handle = h
        .device_operation_async(fuel, DeviceOperation::SleepFor(2_000))
        .unwrap();
    assert_eq!(handle.ready_at, 2_000);

    assert!(matches!(
        h.device_poll(fuel, handle).unwrap(),
        PollOperationStatus::Pending
    ));

    // Closing the round with nothing ready lets the host fast-forward.
    h.poll_loop(fuel, false).unwrap();
    assert_eq!(h.current_time().unwrap(), 2_000);

    // Next round: the future has matured.
    h.poll_loop(fuel, true).unwrap();
    assert!(matches!(
        h.device_poll(fuel, handle).unwrap(),
        PollOperationStatus::Ready(_)
    ));
}

#[test]
fn fast_forward_costs_the_guest_no_fuel() {
    let mut h = host();
    let fuel = fuel_at(0);

    h.poll_loop(fuel, true).unwrap();
    let handle = h
        .device_operation_async(fuel, DeviceOperation::SleepFor(20_000))
        .unwrap();
    h.device_poll(fuel, handle).unwrap();
    h.poll_loop(fuel, false).unwrap();

    // 20 ms of simulated time passed while the guest executed no instructions.
    // Without this, an awaiting robot would spin and burn its entire budget.
    assert_eq!(h.current_time().unwrap(), 20_000);
    assert_eq!(h.current_fuel, fuel);
}

#[test]
fn a_ready_future_suppresses_the_fast_forward() {
    let mut h = host();
    let fuel = fuel_at(0);

    h.poll_loop(fuel, true).unwrap();
    let ready_now = h
        .device_operation_async(fuel, DeviceOperation::GetTime)
        .unwrap();
    let later = h
        .device_operation_async(fuel, DeviceOperation::SleepFor(5_000))
        .unwrap();

    // A fresh round, so `update_futures` matures the one that is already due.
    h.poll_loop(fuel, true).unwrap();
    assert!(matches!(
        h.device_poll(fuel, ready_now).unwrap(),
        PollOperationStatus::Ready(_)
    ));
    assert!(matches!(
        h.device_poll(fuel, later).unwrap(),
        PollOperationStatus::Pending
    ));

    // The guest made progress this round, so time must not jump: it has real work
    // to do before it waits again.
    h.poll_loop(fuel, false).unwrap();
    assert_eq!(h.current_time().unwrap(), 0);
}

#[test]
fn polling_a_consumed_future_is_an_error() {
    let mut h = host();
    let fuel = fuel_at(0);

    h.poll_loop(fuel, true).unwrap();
    let handle = h
        .device_operation_async(fuel, DeviceOperation::GetTime)
        .unwrap();
    h.poll_loop(fuel, true).unwrap();

    assert!(matches!(
        h.device_poll(fuel, handle).unwrap(),
        PollOperationStatus::Ready(_)
    ));
    // Polling again is a bug in the guest's executor, not a tolerable condition.
    assert!(h.device_poll(fuel, handle).is_err());
}

#[test]
fn forgetting_a_handle_cancels_the_operation() {
    let mut h = host();
    let fuel = fuel_at(0);

    h.poll_loop(fuel, true).unwrap();
    let handle = h
        .device_operation_async(fuel, DeviceOperation::SleepFor(5_000))
        .unwrap();
    h.forget_handle(handle);

    // Dropping the future on the guest side removes it from every host index.
    assert!(h.device_poll(fuel, handle).is_err());
}
