//! Детерминированный ГПСЧ (PCG32). Состояние сериализуется вместе с партией,
//! поэтому продолжение сохранения даёт ту же последовательность.

use serde::{Deserialize, Serialize};

const MULTIPLIER: u64 = 6364136223846793005;
const INCREMENT: u64 = 1442695040888963407;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pcg32 {
    state: u64,
}

impl Pcg32 {
    pub fn seeded(seed: u64) -> Self {
        let mut rng = Self { state: 0 };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    /// Сид из системного времени — для обычных партий.
    pub fn from_entropy() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x5eed);
        Self::seeded(nanos ^ (std::process::id() as u64).rotate_left(32))
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Равномерное число в `lo..=hi`.
    pub fn range_inclusive(&mut self, lo: usize, hi: usize) -> usize {
        debug_assert!(lo <= hi);
        let span = (hi - lo) as u64 + 1;
        // Отбрасывание хвоста убирает смещение по модулю.
        let zone = u64::from(u32::MAX) + 1 - (u64::from(u32::MAX) + 1) % span;
        loop {
            let value = u64::from(self.next_u32());
            if value < zone {
                return lo + (value % span) as usize;
            }
        }
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.range_inclusive(0, i);
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Pcg32::seeded(42);
        let mut b = Pcg32::seeded(42);
        for _ in 0..100 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn range_stays_inside_bounds() {
        let mut rng = Pcg32::seeded(7);
        for _ in 0..1000 {
            let v = rng.range_inclusive(2, 5);
            assert!((2..=5).contains(&v));
        }
    }
}
