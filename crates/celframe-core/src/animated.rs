//! Values that change over time. See `docs/specs/keyframes.md`.

use serde::{Deserialize, Serialize};

/// How a key's value travels to the next key's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Interpolation {
    Linear,
    /// Slow out of this key and into the next, by smoothstep.
    #[default]
    Ease,
    /// Hold this key's value until the next key.
    Step,
}

/// A value at a frame.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    pub frame: u32,
    pub value: f64,
    /// How the value travels from this key to the next.
    pub interpolation: Interpolation,
}

/// A value with an optional set of keys.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Animated {
    /// The value while there are no keys.
    pub value: f64,
    /// Sorted by frame, no two at the same frame.
    pub keys: Vec<Key>,
}

impl Animated {
    pub const fn constant(value: f64) -> Self {
        Self {
            value,
            keys: Vec::new(),
        }
    }

    /// The value at a frame.
    pub fn at(&self, frame: u32) -> f64 {
        let (Some(first), Some(last)) = (self.keys.first(), self.keys.last()) else {
            return self.value;
        };
        if frame <= first.frame {
            return first.value;
        }
        if frame >= last.frame {
            return last.value;
        }
        let next = self.keys.partition_point(|k| k.frame <= frame);
        let (a, b) = (self.keys[next - 1], self.keys[next]);
        let t = f64::from(frame - a.frame) / f64::from(b.frame - a.frame);
        let t = match a.interpolation {
            Interpolation::Linear => t,
            Interpolation::Ease => t * t * (3.0 - 2.0 * t),
            Interpolation::Step => 0.0,
        };
        a.value + (b.value - a.value) * t
    }

    /// The key at a frame, if there is one.
    pub fn key_at(&self, frame: u32) -> Option<&Key> {
        self.keys.iter().find(|k| k.frame == frame)
    }

    /// Adds or replaces the key at `key.frame`. Returns the key it replaced.
    pub fn set_key(&mut self, key: Key) -> Option<Key> {
        let index = self.keys.partition_point(|k| k.frame < key.frame);
        match self.keys.get_mut(index) {
            Some(existing) if existing.frame == key.frame => Some(std::mem::replace(existing, key)),
            _ => {
                self.keys.insert(index, key);
                None
            }
        }
    }

    /// Removes and returns the key at a frame.
    pub fn remove_key(&mut self, frame: u32) -> Option<Key> {
        let index = self.keys.iter().position(|k| k.frame == frame)?;
        Some(self.keys.remove(index))
    }

    /// True when keys are sorted with no repeated frames and every value is in `range`.
    pub fn is_valid(&self, range: std::ops::RangeInclusive<f64>) -> bool {
        range.contains(&self.value)
            && self.keys.iter().all(|k| range.contains(&k.value))
            && self.keys.windows(2).all(|w| w[0].frame < w[1].frame)
    }
}
