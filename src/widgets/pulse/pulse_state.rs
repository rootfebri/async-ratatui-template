use super::*;
use std::time::{Duration, Instant};
use strum::EnumIs;

#[derive(Debug, Clone, Eq, PartialEq, Hash, EnumIs)]
enum StepDir {
  Right,
  Left,
}

impl StepDir {
  fn flip(&mut self) {
    match self {
      Self::Right => *self = Self::Left,
      Self::Left => *self = Self::Right,
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PulseState {
  fps: PulseFps,
  steps: usize,
  step_dir: StepDir,
  last_update: Instant,
}

impl PulseState {
  pub fn new(fps: PulseFps) -> Self {
    Self {
      fps,
      steps: 0,
      step_dir: StepDir::Right,
      last_update: Instant::now(),
    }
  }

  pub fn fps(mut self, fps: PulseFps) -> Self {
    self.fps = fps;
    self
  }

  pub fn color(&mut self, level: PulseLevel) -> Color {
    let colors = level.as_colors();
    let total_steps = colors.len() - 1;

    if self.last_update.elapsed() >= Duration::from_secs_f32(1.0 / self.fps as u8 as f32) {
      match self.step_dir {
        StepDir::Right => {
          if self.steps >= total_steps {
            self.steps -= 1;
            self.step_dir.flip();
          } else {
            self.steps += 1;
          }
        }
        StepDir::Left => {
          if self.steps == 0 {
            self.steps += 1;
            self.step_dir.flip();
          } else {
            self.steps -= 1;
          }
        }
      }

      self.last_update = Instant::now();
    }

    colors[self.steps.saturating_sub(1)]
  }
}

impl Default for PulseState {
  fn default() -> Self {
    Self {
      fps: Default::default(),
      steps: Default::default(),
      step_dir: StepDir::Right,
      last_update: Instant::now(),
    }
  }
}
