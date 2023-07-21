pub fn to_celcuis(temperature: f32) -> f32 {
    (temperature - 32.0) * (5.0 / 9.0)
}

pub fn to_fahrenhiet(temperature: f32) -> f32 {
    32.0 + (9.0 / 5.0) * temperature
}