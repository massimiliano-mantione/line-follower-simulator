pub mod async_api;
pub mod async_framework;
pub mod blocking_api;
pub mod examples;
#[allow(warnings)]
pub mod wasm_bindings;
pub mod wasm_bindings_ext;

use wasm_bindings::exports::robot::{Color, Configuration, Guest};

struct Component;

impl Guest for Component {
    fn setup() -> Configuration {
        Configuration {
            name: "Liner".to_string(),
            color_main: Color { r: 255, g: 0, b: 0 },
            color_secondary: Color { r: 0, g: 255, b: 0 },
            width_axle: 100.0,
            length_front: 100.0,
            length_back: 20.0,
            clearing_back: 3.0,
            wheel_diameter: 25.0,
            gear_ratio_num: 1,
            gear_ratio_den: 20,
            front_sensors_spacing: 4.0,
            front_sensors_height: 4.0,
        }
    }

    fn run() -> () {
        // async_framework::run(examples::nb::toy::toy_run());
        // examples::toy::toy_run();
        // examples::basic_pid::basic_pid_run(4.0);
        examples::pid_with_memory::pid_with_memory_run(4.0);
        // examples::telemetry_test::telemetry_test_run();
    }
}

wasm_bindings::export!(Component with_types_in wasm_bindings);
