use wasmtime::component::bindgen;

use crate::{
    motors::MotorPower,
    sensors::{
        AsyncSensorResult, AsyncVoidResult, FutureHandle, ReadError, SensorIndexRange, SensorKind,
        SensorValues, TimeUs,
    },
};

bindgen!({
    path: "../sample-robot-code/wit/"
});

pub struct BotHost {}

impl sensors::Host for BotHost {
    fn current_time(&mut self) -> TimeUs {
        todo!()
    }

    fn sleep_blocking_for(&mut self, _us: TimeUs) -> () {
        todo!()
    }

    fn sleep_blocking_until(&mut self, _us: TimeUs) -> () {
        todo!()
    }

    fn read_sensor_blocking(
        &mut self,
        _sensor: SensorKind,
        _indexes: SensorIndexRange,
    ) -> Result<SensorValues, ReadError> {
        todo!()
    }

    fn sleep_async_for(&mut self, _us: TimeUs) -> FutureHandle {
        todo!()
    }

    fn sleep_async_until(&mut self, _us: TimeUs) -> FutureHandle {
        todo!()
    }

    fn read_sensor_async(
        &mut self,
        _sensor: SensorKind,
        _sensor_index_range: SensorIndexRange,
    ) -> FutureHandle {
        todo!()
    }

    fn poll_timer(&mut self, _handle: FutureHandle) -> Result<AsyncVoidResult, ReadError> {
        todo!()
    }

    fn poll_sensor(&mut self, _handle: FutureHandle) -> Result<AsyncSensorResult, ReadError> {
        todo!()
    }

    fn forget_handle(&mut self, _handle: FutureHandle) -> () {
        todo!()
    }
}

impl motors::Host for BotHost {
    fn set_power(&mut self, _left: MotorPower, _right: MotorPower) -> () {
        todo!()
    }
}

fn main() {
    println!("Hello, world!");
}
