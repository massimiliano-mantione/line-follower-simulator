#[macro_use]
extern crate std;

pub mod bindings;

use crate::bindings::{
    LineFollowerRobot,
    devices::{
        DeviceOperation, DeviceValue, FutureHandle, MotorPower, PollError, PollOperationStatus,
    },
    diagnostics::CsvColumn,
};

pub trait DeviceValueExt {
    fn get_u8(&self, index: usize) -> u8;
    fn get_u16(&self, index: usize) -> u16;
    fn get_i16(&self, index: usize) -> i16;
    fn get_u32(&self, index: usize) -> u32;
    fn get_bool(&self, index: usize) -> bool;

    fn set_u8(self, value: u8, index: usize) -> Self;
    fn set_u16(self, value: u16, index: usize) -> Self;
    fn set_i16(self, value: i16, index: usize) -> Self;
    fn set_u32(self, value: u32, index: usize) -> Self;
    fn set_bool(self, value: bool, index: usize) -> Self;
}

pub const DEVICE_VALUE_ZERO: DeviceValue = DeviceValue {
    v0: 0,
    v1: 0,
    v2: 0,
    v3: 0,
    v4: 0,
    v5: 0,
    v6: 0,
    v7: 0,
};

fn build_u16_from_u8s(v0: u8, v1: u8) -> u16 {
    ((v1 as u16) << 8) | (v0 as u16)
}
fn build_i16_from_u8s(v0: u8, v1: u8) -> i16 {
    build_u16_from_u8s(v0, v1) as i16
}
fn build_u32_from_u8s(v0: u8, v1: u8, v2: u8, v3: u8) -> u32 {
    ((v3 as u32) << 24) | ((v2 as u32) << 16) | ((v1 as u32) << 8) | (v0 as u32)
}

fn decompose_u16_to_u8s(value: u16) -> (u8, u8) {
    let v0 = (value & 0x00FF) as u8;
    let v1 = ((value & 0xFF00) >> 8) as u8;
    (v0, v1)
}
fn decompose_i16_to_u8s(value: i16) -> (u8, u8) {
    decompose_u16_to_u8s(value as u16)
}
fn decompose_u32_to_u8s(value: u32) -> (u8, u8, u8, u8) {
    let v0 = (value & 0x000000FF) as u8;
    let v1 = ((value & 0x0000FF00) >> 8) as u8;
    let v2 = ((value & 0x00FF0000) >> 16) as u8;
    let v3 = ((value & 0xFF000000) >> 24) as u8;
    (v0, v1, v2, v3)
}

impl DeviceValueExt for DeviceValue {
    fn get_u8(&self, index: usize) -> u8 {
        match index {
            0 => self.v0,
            1 => self.v1,
            2 => self.v2,
            3 => self.v3,
            4 => self.v4,
            5 => self.v5,
            6 => self.v6,
            7 => self.v7,
            _ => 0,
        }
    }

    fn get_u16(&self, index: usize) -> u16 {
        match index {
            0 => build_u16_from_u8s(self.v0, self.v1),
            1 => build_u16_from_u8s(self.v2, self.v3),
            2 => build_u16_from_u8s(self.v4, self.v5),
            3 => build_u16_from_u8s(self.v6, self.v7),
            _ => 0,
        }
    }

    fn get_i16(&self, index: usize) -> i16 {
        match index {
            0 => build_i16_from_u8s(self.v0, self.v1),
            1 => build_i16_from_u8s(self.v2, self.v3),
            2 => build_i16_from_u8s(self.v4, self.v5),
            3 => build_i16_from_u8s(self.v6, self.v7),
            _ => 0,
        }
    }

    fn get_u32(&self, index: usize) -> u32 {
        match index {
            0 => build_u32_from_u8s(self.v0, self.v1, self.v2, self.v3),
            1 => build_u32_from_u8s(self.v4, self.v5, self.v6, self.v7),
            _ => 0,
        }
    }

    fn get_bool(&self, index: usize) -> bool {
        match index {
            0 => {
                if self.v0 == 0 {
                    false
                } else {
                    true
                }
            }
            1 => {
                if self.v1 == 0 {
                    false
                } else {
                    true
                }
            }
            2 => {
                if self.v2 == 0 {
                    false
                } else {
                    true
                }
            }
            3 => {
                if self.v3 == 0 {
                    false
                } else {
                    true
                }
            }
            4 => {
                if self.v4 == 0 {
                    false
                } else {
                    true
                }
            }
            5 => {
                if self.v5 == 0 {
                    false
                } else {
                    true
                }
            }
            6 => {
                if self.v6 == 0 {
                    false
                } else {
                    true
                }
            }
            7 => {
                if self.v7 == 0 {
                    false
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    fn set_u8(mut self, value: u8, index: usize) -> Self {
        match index {
            0 => self.v0 = value,
            1 => self.v1 = value,
            2 => self.v2 = value,
            3 => self.v3 = value,
            4 => self.v4 = value,
            5 => self.v5 = value,
            6 => self.v6 = value,
            7 => self.v7 = value,
            _ => (),
        };
        self
    }

    fn set_u16(mut self, value: u16, index: usize) -> Self {
        match index {
            0 => {
                let (v0, v1) = decompose_u16_to_u8s(value);
                self.v0 = v0;
                self.v1 = v1;
            }
            1 => {
                let (v0, v1) = decompose_u16_to_u8s(value);
                self.v2 = v0;
                self.v3 = v1;
            }
            2 => {
                let (v0, v1) = decompose_u16_to_u8s(value);
                self.v4 = v0;
                self.v5 = v1;
            }
            3 => {
                let (v0, v1) = decompose_u16_to_u8s(value);
                self.v6 = v0;
                self.v7 = v1;
            }
            _ => (),
        };
        self
    }

    fn set_i16(mut self, value: i16, index: usize) -> Self {
        match index {
            0 => {
                let (v0, v1) = decompose_i16_to_u8s(value);
                self.v0 = v0;
                self.v1 = v1;
            }
            1 => {
                let (v0, v1) = decompose_i16_to_u8s(value);
                self.v2 = v0;
                self.v3 = v1;
            }
            2 => {
                let (v0, v1) = decompose_i16_to_u8s(value);
                self.v4 = v0;
                self.v5 = v1;
            }
            3 => {
                let (v0, v1) = decompose_i16_to_u8s(value);
                self.v6 = v0;
                self.v7 = v1;
            }
            _ => (),
        };
        self
    }

    fn set_u32(mut self, value: u32, index: usize) -> Self {
        match index {
            0 => {
                let (v0, v1, v2, v3) = decompose_u32_to_u8s(value);
                self.v0 = v0;
                self.v1 = v1;
                self.v2 = v2;
                self.v3 = v3;
            }
            1 => {
                let (v0, v1, v2, v3) = decompose_u32_to_u8s(value);
                self.v4 = v0;
                self.v5 = v1;
                self.v6 = v2;
                self.v7 = v3;
            }
            _ => (),
        };
        self
    }

    fn set_bool(mut self, value: bool, index: usize) -> Self {
        match index {
            0 => self.v0 = if value { 1 } else { 0 },
            1 => self.v1 = if value { 1 } else { 0 },
            2 => self.v2 = if value { 1 } else { 0 },
            3 => self.v3 = if value { 1 } else { 0 },
            4 => self.v4 = if value { 1 } else { 0 },
            5 => self.v5 = if value { 1 } else { 0 },
            6 => self.v6 = if value { 1 } else { 0 },
            7 => self.v7 = if value { 1 } else { 0 },
            _ => (),
        };
        self
    }
}

pub struct BotHost {}

impl bindings::devices::Host for BotHost {
    #[doc = " Perform a blocking operation (returns the provided value, blocking for the needed time)"]
    fn device_operation_blocking(&mut self, _operation: DeviceOperation) -> DeviceValue {
        DEVICE_VALUE_ZERO.set_u32(0, 0)
    }

    #[doc = " Initiate and async operation (immediately returns a handle to the future value)"]
    fn device_operation_async(&mut self, _operation: DeviceOperation) -> FutureHandle {
        0
    }

    #[doc = " Poll the status of an async operation (returns immediately)"]
    fn device_poll(&mut self, _handle: FutureHandle) -> Result<PollOperationStatus, PollError> {
        Ok(PollOperationStatus::Pending)
    }

    #[doc = " Wait for an async operation (returns when ready with the result or immediately with an error)"]
    fn device_wait(&mut self, _handle: FutureHandle) -> Result<DeviceValue, PollError> {
        Ok(DEVICE_VALUE_ZERO)
    }

    #[doc = " Instructs the simulation to forget the handle to an async operation"]
    #[doc = " (is equivalent to dropping the future in Rust)"]
    fn forget_handle(&mut self, _handle: FutureHandle) -> () {}

    #[doc = " Set the power of both motors"]
    fn set_motors_power(&mut self, _left: MotorPower, _right: MotorPower) -> () {}
}

impl bindings::diagnostics::Host for BotHost {
    #[doc = " Write a line of text as a log, like writing to a serial line"]
    #[doc = " (each character takes 100 microseconds)"]
    fn write_line(&mut self, _text: wasmtime::component::__internal::String) -> () {}

    #[doc = " Write a buffer into a file, eventually converting it to CSV"]
    #[doc = " (each byte takes 10 microseconds)"]
    fn write_file(
        &mut self,
        _name: wasmtime::component::__internal::String,
        _data: wasmtime::component::__internal::Vec<u8>,
        _csv: Option<wasmtime::component::__internal::Vec<CsvColumn>>,
    ) -> () {
    }
}

fn main() -> wasmtime::Result<()> {
    // Instantiate the engine and store
    let engine = wasmtime::Engine::default();
    let mut store = wasmtime::Store::new(&engine, BotHost {});

    // Load the component from disk
    let bytes = std::fs::read("../bot/target/wasm32-wasip1/release/line_follower_robot.wasm")?;
    let component = wasmtime::component::Component::new(&engine, bytes)?;

    // Configure the linker
    let mut linker = wasmtime::component::Linker::new(&engine);

    // Ignore unknown imports
    linker.define_unknown_imports_as_traps(&component)?;

    // Instantiate component host
    let robot_component = LineFollowerRobot::instantiate(&mut store, &component, &linker)?;

    let config = robot_component.robot().call_setup(&mut store)?;

    println!("Robot configuration: {:#?}", config);

    Ok(())
}
