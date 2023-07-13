#![allow(unused)]

use std::fs::File;
use std::io::{self, BufReader, Read};
use std::{cell::Cell, path::Path};

#[derive(Debug)]
pub struct XORShiftState {
    val: Cell<usize>,
}

impl XORShiftState {
    pub fn new(seed: usize) -> Self {
        Self {
            val: seed.into()
        }
    }
}

// Random number generator based on a seeded value
// Algorithm Paper: https://www.jstatsoft.org/article/view/v008i14
fn xor_shift32(state: &XORShiftState) -> usize {
    // Get seeded state
    let mut x = state.val.get();

    // XOR rand shift (we shift and XOR the result to "spread" the bits and mimic randomness)
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;

    // Replace old state with randomly generated value, `x`
    state.val.set(x);

    x
}

fn rand_between(min: usize, max: usize, state: &mut XORShiftState) -> usize {
    // Add one to make the range inclusive
    let range = max - min + 1;

    // Generate the rand number. Also, we ensure it is bounded via modulus
    let random = xor_shift32(state) % range;

    // Random is an offset of the min in the range
    let result = min + random;

    // Postcondition: Make sure result is in range
    assert!(
        result >= min && result <= max,
        "Maybe bug: generated rand is not in range"
    );

    result
}

#[derive(Debug)]
pub struct Fuzzer {
    // Raw bytes into a file.
    file_raw: Vec<u8>,

    // Random number seed
    rand_state: XORShiftState,
}

impl Fuzzer {
    pub fn new(file: String, rand_state: XORShiftState) -> Self {


        Self {
            file_raw: file.into_bytes(),
            rand_state,
        }
    }

    pub fn fuzz(&mut self) -> String {
        const MUT_COUNT: usize = 1;

        for _ in 0..MUT_COUNT
        {
            // Get position in file that we will mutate
            let file_begin = 0;
            let file_end = self.file_raw.len() - 1;
            let mut_pos = rand_between(file_begin, file_end, &mut self.rand_state);

            // NOTE: Ok to cast to `u8` since we range is from 0..255
            let rand_val = rand_between(0, 255, &mut self.rand_state) as u8;
            self.file_raw[mut_pos] = rand_val;
        }

        dbg!(String::from_utf8_lossy(&self.file_raw).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> io::Result<()> {
        let seed = 1;

        let xor_state = XORShiftState {
            val: Cell::new(seed),
        };

        //let mut fuzzer = Fuzzer::new(file, xor_state)?;
        //fuzzer.fuzz();

        //dbg!(fuzzer);

        Ok(())
    }
}
