use crate::error::Result;
use std::time::Duration;

pub struct MotionPlanner {
    max_speed: f32,
    max_acceleration: f32,
}

impl MotionPlanner {
    pub fn new(max_speed: f32, max_acceleration: f32) -> Self {
        Self {
            max_speed,
            max_acceleration,
        }
    }

    pub fn interpolate(&self, start: f32, end: f32, t: f32) -> f32 {
        start + (end - start) * t
    }

    pub fn interpolate_trajectory(
        &self,
        start: &[f32],
        end: &[f32],
        steps: usize,
    ) -> Vec<Vec<f32>> {
        let mut trajectory = Vec::new();

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let pose: Vec<f32> = start
                .iter()
                .zip(end.iter())
                .map(|(&s, &e)| self.interpolate(s, e, t))
                .collect();
            trajectory.push(pose);
        }

        trajectory
    }

    pub fn smooth_step(&self, t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }

    pub fn estimate_duration(&self, start: &[f32], end: &[f32]) -> Duration {
        let max_delta = start
            .iter()
            .zip(end.iter())
            .map(|(&s, &e)| (e - s).abs())
            .fold(0.0f32, f32::max);

        let accel_time = self.max_speed / self.max_acceleration;
        let accel_distance = 0.5 * self.max_acceleration * accel_time * accel_time;

        let time_seconds = if max_delta <= 2.0 * accel_distance {
            (2.0 * max_delta / self.max_acceleration).sqrt()
        } else {
            2.0 * accel_time + (max_delta - 2.0 * accel_distance) / self.max_speed
        };

        Duration::from_secs_f32(time_seconds)
    }

    pub fn generate_velocity_profile(&self, distance: f32, steps: usize) -> Vec<f32> {
        let mut profile = Vec::with_capacity(steps);
        let accel_time = self.max_speed / self.max_acceleration;
        let total_time = self.estimate_duration(&[0.0], &[distance]).as_secs_f32();
        
        for i in 0..steps {
            let t = (i as f32 / (steps - 1) as f32) * total_time;
            let velocity = if t < accel_time {
                self.max_acceleration * t
            } else if t > total_time - accel_time {
                self.max_acceleration * (total_time - t)
            } else {
                self.max_speed
            };
            profile.push(velocity);
        }
        
        profile
    }

    pub fn calculate_step_count(&self, start: &[f32], end: &[f32], step_size: f32) -> usize {
        let max_delta = start
            .iter()
            .zip(end.iter())
            .map(|(&s, &e)| (e - s).abs())
            .fold(0.0f32, f32::max);

        ((max_delta / step_size).ceil() as usize).max(1)
    }
}

impl Default for MotionPlanner {
    fn default() -> Self {
        Self::new(90.0, 180.0)
    }
}

pub struct TrajectoryPoint {
    pub pose: Vec<f32>,
    pub timestamp: Duration,
}

pub struct Trajectory {
    pub points: Vec<TrajectoryPoint>,
}

impl Trajectory {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    pub fn add_point(&mut self, pose: Vec<f32>, timestamp: Duration) {
        self.points.push(TrajectoryPoint { pose, timestamp });
    }

    pub fn interpolate_at(&self, time: Duration) -> Result<Vec<f32>> {
        if self.points.is_empty() {
            return Ok(Vec::new());
        }

        if self.points.len() == 1 {
            return Ok(self.points[0].pose.clone());
        }

        for i in 0..self.points.len() - 1 {
            let p1 = &self.points[i];
            let p2 = &self.points[i + 1];

            if time >= p1.timestamp && time <= p2.timestamp {
                let dt = (p2.timestamp - p1.timestamp).as_secs_f32();
                let t = (time - p1.timestamp).as_secs_f32() / dt;

                return Ok(p1
                    .pose
                    .iter()
                    .zip(p2.pose.iter())
                    .map(|(&a, &b)| a + (b - a) * t)
                    .collect());
            }
        }

        Ok(self.points.last().unwrap().pose.clone())
    }
}

impl Default for Trajectory {
    fn default() -> Self {
        Self::new()
    }
}
