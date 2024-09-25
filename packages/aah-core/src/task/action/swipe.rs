use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{task::Runnable, AAH};

use super::Action;

mod duration_as_sec_f32 {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f32(duration.as_secs_f32())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = f32::deserialize(deserializer)?;
        Ok(Duration::from_secs_f32(s))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swipe {
    p1: (u32, u32),
    p2: (i32, i32),
    #[serde(with = "duration_as_sec_f32")]
    duration: Duration,
    slope_in: f32,
    slope_out: f32,
}

impl Into<Action> for Swipe {
    fn into(self) -> Action {
        Action::ActionSwipe(self)
    }
}

impl Swipe {
    pub fn new(
        p1: (u32, u32),
        p2: (i32, i32),
        duration: Duration,
        slope_in: f32,
        slope_out: f32,
    ) -> Self {
        Self {
            p1,
            p2,
            duration,
            slope_in,
            slope_out,
        }
    }
}

impl Runnable for Swipe {
    type Err = String;
    fn run(&self, aah: &AAH) -> Result<Self::Res, Self::Err> {
        aah.controller
            .swipe(
                self.p1,
                self.p2,
                self.duration,
                self.slope_in,
                self.slope_out,
            )
            .map_err(|err| format!("controller error: {:?}", err))
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::Swipe;

    #[test]
    fn test_ser() {
        let swipe = Swipe {
            p1: (10, 10),
            p2: (20, 20),
            duration: Duration::from_secs_f32(0.5),
            slope_in: 0.1,
            slope_out: 1.0,
        };
        let swipe = toml::to_string(&swipe).unwrap();
        println!("{}", swipe);
    }
}
