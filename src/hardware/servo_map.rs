use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Finger {
    Thumb,
    Index,
    Middle,
    Ring,
    Pinky,
}

impl Finger {
    pub fn all() -> [Finger; 5] {
        [
            Finger::Thumb,
            Finger::Index,
            Finger::Middle,
            Finger::Ring,
            Finger::Pinky,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Finger::Thumb => "Thumb",
            Finger::Index => "Index",
            Finger::Middle => "Middle",
            Finger::Ring => "Ring",
            Finger::Pinky => "Pinky",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ServoConfig {
    pub id: u8,
    pub inverted: bool,
    pub min_angle: f32,
    pub max_angle: f32,
}

impl ServoConfig {
    pub fn new(id: u8, inverted: bool) -> Self {
        Self {
            id,
            inverted,
            min_angle: 0.0,
            max_angle: 180.0,
        }
    }

    pub fn with_limits(mut self, min_angle: f32, max_angle: f32) -> Self {
        self.min_angle = min_angle;
        self.max_angle = max_angle;
        self
    }

    pub fn translate_angle(&self, angle: f32) -> f32 {
        let clamped = angle.clamp(self.min_angle, self.max_angle);
        if self.inverted {
            180.0 - clamped
        } else {
            clamped
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServoMap {
    map: HashMap<Finger, ServoConfig>,
}

impl ServoMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn hardware_default() -> Self {
        let mut map = HashMap::new();

        map.insert(Finger::Ring, ServoConfig::new(1, false));
        map.insert(Finger::Middle, ServoConfig::new(2, false));
        map.insert(Finger::Pinky, ServoConfig::new(3, false));
        map.insert(Finger::Index, ServoConfig::new(4, true));
        map.insert(Finger::Thumb, ServoConfig::new(5, false));

        Self { map }
    }

    pub fn simple_mapping() -> Self {
        let mut map = HashMap::new();

        map.insert(Finger::Thumb, ServoConfig::new(0, false));
        map.insert(Finger::Index, ServoConfig::new(1, false));
        map.insert(Finger::Middle, ServoConfig::new(2, false));
        map.insert(Finger::Ring, ServoConfig::new(3, false));
        map.insert(Finger::Pinky, ServoConfig::new(4, false));

        Self { map }
    }

    pub fn insert(&mut self, finger: Finger, config: ServoConfig) {
        self.map.insert(finger, config);
    }

    pub fn get(&self, finger: Finger) -> Option<&ServoConfig> {
        self.map.get(&finger)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&ServoConfig> {
        let finger = match name.to_lowercase().as_str() {
            "thumb" => Finger::Thumb,
            "index" | "pointer" => Finger::Index,
            "middle" => Finger::Middle,
            "ring" => Finger::Ring,
            "pinky" | "left" => Finger::Pinky,
            _ => return None,
        };
        self.get(finger)
    }

    pub fn translate_angle(&self, finger: Finger, angle: f32) -> Option<f32> {
        self.get(finger).map(|config| config.translate_angle(angle))
    }

    pub fn get_servo_id(&self, finger: Finger) -> Option<u8> {
        self.get(finger).map(|config| config.id)
    }

    pub fn get_servo_id_by_name(&self, name: &str) -> Option<u8> {
        self.get_by_name(name).map(|config| config.id)
    }

    pub fn to_legacy_map(&self) -> HashMap<String, u8> {
        let mut legacy = HashMap::new();
        for finger in Finger::all() {
            if let Some(config) = self.get(finger) {
                legacy.insert(finger.name().to_string(), config.id);
            }
        }
        legacy
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Finger, &ServoConfig)> {
        self.map.iter()
    }
}

impl Default for ServoMap {
    fn default() -> Self {
        Self::hardware_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_default_mapping() {
        let servo_map = ServoMap::hardware_default();

        assert_eq!(servo_map.get_servo_id(Finger::Ring), Some(1));
        assert_eq!(servo_map.get_servo_id(Finger::Middle), Some(2));
        assert_eq!(servo_map.get_servo_id(Finger::Pinky), Some(3));
        assert_eq!(servo_map.get_servo_id(Finger::Index), Some(4));
    }

    #[test]
    fn test_inverted_servo() {
        let servo_map = ServoMap::hardware_default();
        let config = servo_map.get(Finger::Index).unwrap();

        assert_eq!(config.translate_angle(0.0), 180.0);
        assert_eq!(config.translate_angle(90.0), 90.0);
        assert_eq!(config.translate_angle(180.0), 0.0);
    }

    #[test]
    fn test_get_by_name() {
        let servo_map = ServoMap::hardware_default();

        assert_eq!(servo_map.get_servo_id_by_name("index"), Some(4));
        assert_eq!(servo_map.get_servo_id_by_name("pointer"), Some(4));
        assert_eq!(servo_map.get_servo_id_by_name("pinky"), Some(3));
    }
}
