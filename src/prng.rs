use std::{u64::MAX, time};

#[test]
fn test_prng_percentage_chance() {
    let mut lcrng = Lcrng::new(seed_from_time_now());
    let mut dist = [0u64; 100];
    for _ in 0..=10_000_000 {
        let n = lcrng.number_in_range_inclusive(0, 99) as usize;
        dist[n] += 1;
    }
    let avg_deviation = dist
        .iter()
        .map(|it| ((*it as f32 / 100_000.0) - 1.0).abs())
        .reduce(|it, acc| { it + acc })
        .expect("We should always get some average value.") / 100.0; 
    println!("LCRNG has {:?}% average deviation", avg_deviation);
   assert!(avg_deviation < 3.0e-3);
}

#[test]
fn test_prng_idempotence() {
    let seed = seed_from_time_now();
    let mut lcrng_1 = Lcrng::new(seed);
    let mut generated_numbers_1 = [0;10_000];
    for i in 0..10_000 {
        generated_numbers_1[i] = lcrng_1.next();
    }
    let mut lcrng_2 = Lcrng::new(seed);
    let mut generated_numbers_2 = [0;10_000];
    for i in 0..10_000 {
        generated_numbers_2[i] = lcrng_2.next();
    }
    assert_eq!(generated_numbers_1, generated_numbers_2);
}

pub fn seed_from_time_now() -> u64 {
    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// #[test]
// fn test_prng_chance() {
//     let mut lcrng = LCRNG::new(
//         time::SystemTime::now()
//             .duration_since(time::UNIX_EPOCH)
//             .unwrap()
//             .as_secs()
//         );

//     let mut success = 0.0;
//     for _ in 0..=10_000_000 {
//         if lcrng.chance(33, 100) {
//             success += 1.0;
//         }
//     }
//     let avg_probability_deviation = (success / 10_000_000.0) - 0.33;
//     println!("Average probability of LCRNG is off by {:?}", avg_probability_deviation);
//    assert!(avg_probability_deviation < 3.0e-3);
// }

// LCRNG -> Linear Congruential Random Number Generator
#[derive(Debug, Clone, Copy)]
pub struct Lcrng { 
    _start_seed: u64,
    current_seed: u64
}

const A: u64 = 0x5D588B656C078965;
const C: u64 = 0x00269EC3;

impl Lcrng {
    pub(crate) fn new(start_seed: u64) -> Self {
        Self {
            _start_seed: start_seed,
            current_seed: start_seed,
        }
    }
    
    fn next(&mut self) -> u64 {
        // x_{n+1} = (A * x_n) + C <- Core function for the LCRNG 
        self.current_seed = self.current_seed.wrapping_mul(A).wrapping_add(C);
        self.current_seed
    } 

    pub(crate) fn number_in_range_inclusive(&mut self, start: u64, end: u64) -> u64 {
        assert!(end >= 1, "The end of the range should be 1 or higher");
        let random_number = self.next();
        let range = (end - start + 1) as f64;

        ((random_number as f64 / MAX as f64) * range) as u64 + start
    }

    pub(crate) fn _chance(&mut self, num: u64, denom: u64) -> bool {
        assert!(denom != 0);
        self.number_in_range_inclusive(1, denom) <= num
    }
}