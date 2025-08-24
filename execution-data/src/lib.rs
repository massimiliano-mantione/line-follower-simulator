pub struct ExecutionStep {
    pub time_us: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub struct ExecutionData {
    pub steps: Vec<ExecutionStep>,
}
