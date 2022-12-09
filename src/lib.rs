//! # ACORN Pseudo-random Number Generator
//!
//! This pseudo-random number generator is based on the [ACORN](http://acorn.wikramaratna.org/concept.html) algorithm.
//!
//! It is a `#![no_std]` crate that only requires [alloc](https://doc.rust-lang.org/alloc/index.html) for using
//! [vectors](https://doc.rust-lang.org/alloc/vec/index.html).
//!
//! The numbers generated from this prng are not considered cryptographically secure.
//!
//! ## Usage
//!
//! Create a generator by specifying both the [Order](struct.Order.html) and the [Seed](struct.Seed.html).
//! Allowing the user of this library to specify the starting data makes reproducability possible.
//! If you do not require reproducable pseudo-random numbers then using the current time converted into
//! a [`u128`](https://doc.rust-lang.org/core/primitive.u128.html) for the [Seed](struct.Seed.html) should provide
//! sufficient randomness.
//!
//! Then you can generate either a number of a fixed digit length or a number bewteen a specified range (inclusive).
//!
//! Please see the [Acorn](struct.Acorn.html) struct documentation for examples.
#![no_std]
extern crate alloc;

use alloc::vec::Vec;

/// The order used for the ACORN algorithm.
#[derive(Debug, Eq, PartialEq)]
pub struct Order(usize);

impl Order {
    /// Create a new [Order](struct.Order.html) for constructing an [Acorn](struct.Acorn.html) generator.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::Order;
    ///
    /// let order = Order::new(45);
    /// ```
    /// Note that the input is clamped between 45 and [`u16::MAX`] rather than rejected.
    ///
    /// [`u16::MAX`]: https://doc.rust-lang.org/core/primitive.u16.html#associatedconstant.MAX
    #[must_use]
    pub fn new(input: usize) -> Self {
        Self(input.clamp(45, 65_535))
    }
}

/// The seed used for the ACORN algorithm.
#[derive(Debug, Eq, PartialEq)]
pub struct Seed(u128);

impl Seed {
    /// Create a new [Seed](struct.Seed.html) for constructing an [Acorn](struct.Acorn.html) generator.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::Seed;
    ///
    /// let seed = Seed::new(1_000_000);
    /// ```
    /// Note that the input is clamped between 1,000,000 and [`u128::MAX`] rather than rejected.
    ///
    /// [`u128::MAX`]: https://doc.rust-lang.org/core/primitive.u128.html#associatedconstant.MAX
    #[must_use]
    pub fn new(input: u128) -> Self {
        Self(input.clamp(1_000_000, 340_282_366_920_938_463_463_374_607_431_768_211_455))
    }
}

#[derive(PartialEq)]
enum NumType {
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
}

/// Additive Congruential Random Number (ACORN) generator.
#[derive(Debug, Eq, PartialEq)]
pub struct Acorn {
    k: Order,
    m: u128,
    y: Vec<u128>,
}

impl Acorn {
    /// Create a new ACORN generator.
    ///
    /// This function always cycles through generating 67 individual [`u128`]s. This ensures that the
    /// modulus ceiling has been passed and wraps back around to generate a pseudo-random number even
    /// when the lowest values are provided for both [Order](struct.Order.html) and [Seed](struct.Seed.html).
    /// It returns an [Acorn](struct.Acorn.html) struct that you can use to generate pseudo-random numbers.
    ///
    /// The internal modulus is fixed as a [`u128`] set to 2^120.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let order = Order::new(45);
    /// let seed = Seed::new(1_000_000);
    /// let mut prng = Acorn::new(order, seed);
    /// ```
    /// Note that the created generator needs to be mutable to change the internal state.
    ///
    /// [`u128`]: https://doc.rust-lang.org/core/primitive.u128.html
    #[must_use]
    pub fn new(k: Order, mut seed: Seed) -> Self {
        if seed.0 % 2 == 0 {seed.0 += 1} // ensure seed is odd
        let m = 2_u128.pow(120); // set modulus to 2^120
        seed.0 %= m; // ensure seed is less than m
        let y = alloc::vec![seed.0; k.0]; // initialise vector of size k with the seed
        let mut acorn = Self {k,m,y};
        (0..67).for_each(|_| {acorn.generate_u128();}); // cycle through the first 67
        acorn
    }
    fn generate_u128(&mut self) -> u128 {
        let mut first = 0;
        let mut second = self.y[0];
        (1..self.k.0).for_each(|index| {
            first = (second + self.y[index]) % self.m;
            self.y[index-1] = second;
            second = first;
        });
        self.y[self.k.0-1] = first;
        first
    }
    /// Generate a random [`usize`] of a fixed digit length.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
    /// let number = prng.generate_fixed_length_usize(3);
    ///
    /// assert_eq!(448, number); // assuming above input. further calls will produce different results
    /// ```
    /// Note that the max size of a [`usize`] is platform dependant.
    ///
    /// [`usize`]: https://doc.rust-lang.org/core/primitive.usize.html
    pub fn generate_fixed_length_usize(&mut self, length: usize) -> usize {
        let number = self.generate_fixed_length_number(length, &NumType::Usize);
        number as usize
    }
    /// Generate a random [`u8`] of a fixed digit length.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
    /// let number = prng.generate_fixed_length_u8(3);
    ///
    /// assert_eq!(192, number); // assuming above input. further calls will produce different results
    /// ```
    /// Note that the `length` is clamped between 1 and 3 because [`u8::MAX`] is 3 digits long.
    ///
    /// [`u8`]: https://doc.rust-lang.org/core/primitive.u8.html
    /// [`u8::MAX`]: https://doc.rust-lang.org/core/primitive.u8.html#associatedconstant.MAX
    #[allow(clippy::cast_possible_truncation)]
    pub fn generate_fixed_length_u8(&mut self, length: usize) -> u8 {
        let length = length.clamp(1, 3);
        let mut number = self.generate_fixed_length_number(length, &NumType::U8);
        while number > 255 {number = self.generate_fixed_length_number(length, &NumType::U8);}
        number as u8
    }
    /// Generate a random [`u16`] of a fixed digit length.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
    /// let number = prng.generate_fixed_length_u16(3);
    ///
    /// assert_eq!(448, number); // assuming above input. further calls will produce different results
    /// ```
    /// Note that the `length` is clamped between 1 and 5 because [`u16::MAX`] is 5 digits long.
    ///
    /// [`u16`]: https://doc.rust-lang.org/core/primitive.u16.html
    /// [`u16::MAX`]: https://doc.rust-lang.org/core/primitive.u16.html#associatedconstant.MAX
    #[allow(clippy::cast_possible_truncation)]
    pub fn generate_fixed_length_u16(&mut self, length: usize) -> u16 {
        let length = length.clamp(1, 5);
        let mut number = self.generate_fixed_length_number(length, &NumType::U16);
        while number > 65_535 {number = self.generate_fixed_length_number(length, &NumType::U16);}
        number as u16
    }
    /// Generate a random [`u32`] of a fixed digit length.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
    /// let number = prng.generate_fixed_length_u32(3);
    ///
    /// assert_eq!(448, number); // assuming above input. further calls will produce different results
    /// ```
    /// Note that the `length` is clamped between 1 and 10 because [`u32::MAX`] is 10 digits long.
    ///
    /// [`u32`]: https://doc.rust-lang.org/core/primitive.u32.html
    /// [`u32::MAX`]: https://doc.rust-lang.org/core/primitive.u32.html#associatedconstant.MAX
    #[allow(clippy::cast_possible_truncation)]
    pub fn generate_fixed_length_u32(&mut self, length: usize) -> u32 {
        let length = length.clamp(1, 10);
        let mut number = self.generate_fixed_length_number(length, &NumType::U32);
        while number > 4_294_967_295 {number = self.generate_fixed_length_number(length, &NumType::U32);}
        number as u32
    }
    /// Generate a random [`u64`] of a fixed digit length.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
    /// let number = prng.generate_fixed_length_u64(3);
    ///
    /// assert_eq!(448, number); // assuming above input. further calls will produce different results
    /// ```
    /// Note that the `length` is clamped between 1 and 20 because [`u64::MAX`] is 20 digits long.
    ///
    /// [`u64`]: https://doc.rust-lang.org/core/primitive.u64.html
    /// [`u64::MAX`]: https://doc.rust-lang.org/core/primitive.u64.html#associatedconstant.MAX
    #[allow(clippy::cast_possible_truncation)]
    pub fn generate_fixed_length_u64(&mut self, length: usize) -> u64 {
        let length = length.clamp(1, 20);
        let mut number = self.generate_fixed_length_number(length, &NumType::U64);
        while number > 18_446_744_073_709_551_615 {number = self.generate_fixed_length_number(length, &NumType::U64);}
        number as u64
    }
    /// Generate a random [`u128`] of a fixed digit length.
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
    /// let number = prng.generate_fixed_length_u128(3);
    ///
    /// assert_eq!(448, number); // assuming above input. further calls will produce different results
    /// ```
    /// Note that the `length` is clamped between 1 and 39 because [`u128::MAX`] is 39 digits long.
    ///
    /// [`u128`]: https://doc.rust-lang.org/core/primitive.u128.html
    /// [`u128::MAX`]: https://doc.rust-lang.org/core/primitive.u128.html#associatedconstant.MAX
    pub fn generate_fixed_length_u128(&mut self, length: usize) -> u128 {
        self.generate_fixed_length_number(length, &NumType::U128)
    }
    fn generate_fixed_length_number(&mut self, length: usize, num_type: &NumType) -> u128 {
        let length = length.clamp(1, 39);
        if length == 1 {return self.generate_index(10) as u128} // allows generating 0
        let (lower_bound, upper_bound) = Acorn::generate_bounds(length, num_type);
        self.generate_number_between_range(lower_bound..=upper_bound)
    }
    /// Generate a random [`usize`] within a given [`RangeInclusive`].
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
    /// let number = prng.generate_usize_between_range(71..=777);
    ///
    /// assert_eq!(419, number); // assuming above input. further calls will produce different results
    /// ```
    ///
    /// [`usize`]: https://doc.rust-lang.org/core/primitive.usize.html
    /// [`RangeInclusive`]: https://doc.rust-lang.org/core/ops/struct.RangeInclusive.html
    pub fn generate_usize_between_range(&mut self, range: core::ops::RangeInclusive<usize>) -> usize {
        let start = *range.start() as u128;
        let end = *range.end() as u128;
        let number = self.generate_number_between_range(start..=end);
        number as usize
    }
    /// Generate a random [`u8`] within a given [`RangeInclusive`].
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
    /// let number = prng.generate_u8_between_range(71..=255);
    ///
    /// assert_eq!(163, number); // assuming above input. further calls will produce different results
    /// ```
    ///
    /// [`u8`]: https://doc.rust-lang.org/core/primitive.u8.html
    /// [`RangeInclusive`]: https://doc.rust-lang.org/core/ops/struct.RangeInclusive.html
    #[allow(clippy::cast_possible_truncation)]
    pub fn generate_u8_between_range(&mut self, range: core::ops::RangeInclusive<u8>) -> u8 {
        let start = u128::from(*range.start());
        let end = u128::from(*range.end());
        let number = self.generate_number_between_range(start..=end);
        number as u8
    }
    /// Generate a random [`u16`] within a given [`RangeInclusive`].
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
    /// let number = prng.generate_u16_between_range(71..=777);
    ///
    /// assert_eq!(419, number); // assuming above input. further calls will produce different results
    /// ```
    ///
    /// [`u16`]: https://doc.rust-lang.org/core/primitive.u16.html
    /// [`RangeInclusive`]: https://doc.rust-lang.org/core/ops/struct.RangeInclusive.html
    #[allow(clippy::cast_possible_truncation)]
    pub fn generate_u16_between_range(&mut self, range: core::ops::RangeInclusive<u16>) -> u16 {
        let start = u128::from(*range.start());
        let end = u128::from(*range.end());
        let number = self.generate_number_between_range(start..=end);
        number as u16
    }
    /// Generate a random [`u32`] within a given [`RangeInclusive`].
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
    /// let number = prng.generate_u32_between_range(71..=777);
    ///
    /// assert_eq!(419, number); // assuming above input. further calls will produce different results
    /// ```
    ///
    /// [`u32`]: https://doc.rust-lang.org/core/primitive.u32.html
    /// [`RangeInclusive`]: https://doc.rust-lang.org/core/ops/struct.RangeInclusive.html
    #[allow(clippy::cast_possible_truncation)]
    pub fn generate_u32_between_range(&mut self, range: core::ops::RangeInclusive<u32>) -> u32 {
        let start = u128::from(*range.start());
        let end = u128::from(*range.end());
        let number = self.generate_number_between_range(start..=end);
        number as u32
    }
    /// Generate a random [`u64`] within a given [`RangeInclusive`].
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
    /// let number = prng.generate_u64_between_range(71..=777);
    ///
    /// assert_eq!(419, number); // assuming above input. further calls will produce different results
    /// ```
    ///
    /// [`u64`]: https://doc.rust-lang.org/core/primitive.u64.html
    /// [`RangeInclusive`]: https://doc.rust-lang.org/core/ops/struct.RangeInclusive.html
    #[allow(clippy::cast_possible_truncation)]
    pub fn generate_u64_between_range(&mut self, range: core::ops::RangeInclusive<u64>) -> u64 {
        let start = u128::from(*range.start());
        let end = u128::from(*range.end());
        let number = self.generate_number_between_range(start..=end);
        number as u64
    }
    /// Generate a random [`u128`] within a given [`RangeInclusive`].
    ///
    /// # Examples
    ///
    /// ```
    /// use acorn_prng::{Acorn, Order, Seed};
    ///
    /// let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
    /// let number = prng.generate_u128_between_range(71..=777);
    ///
    /// assert_eq!(419, number); // assuming above input. further calls will produce different results
    /// ```
    ///
    /// [`u128`]: https://doc.rust-lang.org/core/primitive.u128.html
    /// [`RangeInclusive`]: https://doc.rust-lang.org/core/ops/struct.RangeInclusive.html
    pub fn generate_u128_between_range(&mut self, range: core::ops::RangeInclusive<u128>) -> u128 {
        self.generate_number_between_range(range)
    }
    fn generate_from_zero_range(&mut self, upper_bound: u128) -> u128 {
        if upper_bound.is_power_of_two() {return self.generate_u128() % upper_bound;}
        let x = if upper_bound > 2_u128.pow(127) {2_u128.pow(127)} else {upper_bound.next_power_of_two()};
        let mut number = self.generate_u128() % x;
        while number > upper_bound {
            number = self.generate_u128() % x;
        }
        number
    }
    fn generate_number_between_range(&mut self, range: core::ops::RangeInclusive<u128>) -> u128 {
        self.generate_from_zero_range(*range.end() - *range.start()) + *range.start()
    }
    fn generate_index(&mut self, upper_bound: usize) -> usize {
        let mut indicies: Vec<usize> = (0..upper_bound).collect();
        while indicies.len() > 1 {
            let mut coin_flips = Vec::with_capacity(indicies.len());
            for _ in &indicies {coin_flips.push(self.coin_flip_is_even());}
            if coin_flips.iter().all(|flip|*flip) | coin_flips.iter().all(|flip|!*flip) {continue;}
            let mut offset = 0;
            coin_flips.into_iter().enumerate().for_each(|(index, heads)| {
                if heads {indicies.remove(index-offset); offset += 1;}
            });
        }
        indicies[0]
    }
    fn coin_flip_is_even(&mut self) -> bool {
        self.generate_u128() % 2 == 0
    }
    fn generate_bounds(length: usize, num_type: &NumType) -> (u128, u128) {
        match length {
            2 => (10,99),
            3 => (100, if *num_type == NumType::U8 {u128::from(u8::MAX)} else {999}),
            4 => (1_000, 9_999),
            5 => (10_000, if *num_type == NumType::U16 {u128::from(u16::MAX)} else {99_999}),
            6 => (100_000, 999_999),
            7 => (1_000_000, 9_999_999),
            8 => (10_000_000, 99_999_999),
            9 => (100_000_000, 999_999_999),
            10 => (1_000_000_000, if *num_type == NumType::U32 {u128::from(u32::MAX)} else {9_999_999_999}),
            11 => (10_000_000_000, 99_999_999_999),
            12 => (100_000_000_000, 999_999_999_999),
            13 => (1_000_000_000_000, 9_999_999_999_999),
            14 => (10_000_000_000_000, 99_999_999_999_999),
            15 => (100_000_000_000_000, 999_999_999_999_999),
            16 => (1_000_000_000_000_000, 9_999_999_999_999_999),
            17 => (10_000_000_000_000_000, 99_999_999_999_999_999),
            18 => (100_000_000_000_000_000, 999_999_999_999_999_999),
            19 => (1_000_000_000_000_000_000, 9_999_999_999_999_999_999),
            20 => (10_000_000_000_000_000_000,
                if *num_type == NumType::U64 {u128::from(u64::MAX)} else {99_999_999_999_999_999_999}),
            21 => (100_000_000_000_000_000_000, 999_999_999_999_999_999_999),
            22 => (1_000_000_000_000_000_000_000, 9_999_999_999_999_999_999_999),
            23 => (10_000_000_000_000_000_000_000, 99_999_999_999_999_999_999_999),
            24 => (100_000_000_000_000_000_000_000, 999_999_999_999_999_999_999_999),
            25 => (1_000_000_000_000_000_000_000_000, 9_999_999_999_999_999_999_999_999),
            26 => (10_000_000_000_000_000_000_000_000, 99_999_999_999_999_999_999_999_999),
            27 => (100_000_000_000_000_000_000_000_000, 999_999_999_999_999_999_999_999_999),
            28 => (1_000_000_000_000_000_000_000_000_000, 9_999_999_999_999_999_999_999_999_999),
            29 => (10_000_000_000_000_000_000_000_000_000, 99_999_999_999_999_999_999_999_999_999),
            30 => (100_000_000_000_000_000_000_000_000_000, 999_999_999_999_999_999_999_999_999_999),
            31 => (1_000_000_000_000_000_000_000_000_000_000, 9_999_999_999_999_999_999_999_999_999_999),
            32 => (10_000_000_000_000_000_000_000_000_000_000, 99_999_999_999_999_999_999_999_999_999_999),
            33 => (100_000_000_000_000_000_000_000_000_000_000, 999_999_999_999_999_999_999_999_999_999_999),
            34 => (1_000_000_000_000_000_000_000_000_000_000_000, 9_999_999_999_999_999_999_999_999_999_999_999),
            35 => (10_000_000_000_000_000_000_000_000_000_000_000, 99_999_999_999_999_999_999_999_999_999_999_999),
            36 => (100_000_000_000_000_000_000_000_000_000_000_000, 999_999_999_999_999_999_999_999_999_999_999_999),
            37 => (1_000_000_000_000_000_000_000_000_000_000_000_000, 9_999_999_999_999_999_999_999_999_999_999_999_999),
            38 => (10_000_000_000_000_000_000_000_000_000_000_000_000, 99_999_999_999_999_999_999_999_999_999_999_999_999),
            39 => (100_000_000_000_000_000_000_000_000_000_000_000_000, u128::MAX),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_order() {
        assert_eq!(Order::new(1), Order(45));
        assert_eq!(Order::new(77), Order(77));
        assert_eq!(Order::new(1_000_000), Order(65_535));
    }
    #[test]
    fn new_seed() {
        assert_eq!(Seed::new(1), Seed(1_000_000));
        assert_eq!(Seed::new(777_777_777), Seed(777_777_777));
        assert_eq!(Seed::new(u128::MAX), Seed(340_282_366_920_938_463_463_374_607_431_768_211_455));
    }
    #[test]
    fn new_acorn() {
        assert_eq!(Acorn::new(Order::new(45), Seed::new(1_000_000)),
            Acorn {
                k: Order(45),
                m: 2_u128.pow(120),
                y: alloc::vec![1000001, 68000068, 2346002346, 54740054740, 971635971635, 13991557991544,
                    170230622230452, 1799580863579064, 16871070596053725, 142466818366675900, 1096994501423404430,
                    7778688282820504140, 51209697861901652255, 315136602227087090800, 1823290341456718168200,
                    9967320533296725986160, 51705475266476766053205, 255485877787296961674660,
                    1206461089551124541241450, 5460823879020879502461300, 23754583873740825835706655,
                    99543018137580603502008840, 402696755192939714167217580, 1575769911624546707610851400,
                    5974794248243072933024478225, 21987242833534508393530079868, 78646676289181126176857593374,
                    273806947080852809652763473228, 928987856167179175607590355595, 3075270144553420719252712901280,
                    9943373467389393658917105047472, 31433890316263244470125041762976,
                    97248598165939412579449347954207, 294692721714967916907422266527900,
                    875410732153287047283813203509350, 2551196990846722252084255621655820,
                    7299258057144788665685509139737485, 20516833457920487060305214879262120,
                    56691250344253977403474935850592700, 154083911192074912942778030773405800,
                    412174462438800392121931232318860515, 1085727852277815667052892026596022820,
                    159266291722594628210605662748036738, 561986792288604382969433911713622420,
                    904174045811170833414601003987414337],
            }
        );
    }
    #[test]
    fn new_u128() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_u128(), 707_329_019_109_624_976_857_103_382_873_185_628);
    }
    #[test]
    fn new_fixed_length_usize() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_fixed_length_usize(3), 448);
    }
    #[test]
    fn new_fixed_length_u8() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_fixed_length_u8(3), 192);
    }
    #[test]
    fn new_fixed_length_u16() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_fixed_length_u16(5), 17_516);
    }
    #[test]
    fn new_fixed_length_u32() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_fixed_length_u32(10), 1_674_307_420);
    }
    #[test]
    fn new_fixed_length_u64() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_fixed_length_u64(20), 11_008_839_946_799_226_204);
    }
    #[test]
    fn new_fixed_length_u128() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_fixed_length_u128(39), 100_707_329_019_109_624_976_857_103_382_873_185_628);
    }
    #[test]
    fn new_fixed_length_number() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_fixed_length_number(3, &NumType::U128), 448);
    }
    #[test]
    fn new_usize_between_range_different_length() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_usize_between_range(71..=777), 419);
    }
    #[test]
    fn new_u8_between_range_different_length() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_u8_between_range(71..=255), 163);
    }
    #[test]
    fn new_u16_between_range_different_length() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_u16_between_range(71..=777), 419);
    }
    #[test]
    fn new_u32_between_range_different_length() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_u32_between_range(71..=777), 419);
    }
    #[test]
    fn new_u64_between_range_different_length() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_u64_between_range(71..=777), 419);
    }
    #[test]
    fn new_u128_between_range_different_length() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_u128_between_range(71..=777), 419);
    }
    #[test]
    fn new_number_between_range_different_length() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_number_between_range(71..=777), 419);
    }
    #[test]
    fn new_number_between_range_same_length() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_number_between_range(750..=777), 762);
    }
    #[test]
    fn new_index() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_index(15), 12);
    }
    #[test]
    fn new_coin_flip() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert!(prng.coin_flip_is_even());
    }
    #[test]
    fn bounds_testing() {
        assert_eq!(Acorn::generate_bounds(2, &NumType::U128), (10, 99));
        assert_eq!(Acorn::generate_bounds(4, &NumType::U128), (1_000, 9_999));
        assert_eq!(Acorn::generate_bounds(6, &NumType::U128), (100_000, 999_999));
        assert_eq!(Acorn::generate_bounds(7, &NumType::U128), (1_000_000, 9_999_999));
        assert_eq!(Acorn::generate_bounds(8, &NumType::U128), (10_000_000, 99_999_999));
        assert_eq!(Acorn::generate_bounds(9, &NumType::U128), (100_000_000, 999_999_999));
        assert_eq!(Acorn::generate_bounds(11, &NumType::U128), (10_000_000_000, 99_999_999_999));
        assert_eq!(Acorn::generate_bounds(12, &NumType::U128), (100_000_000_000, 999_999_999_999));
        assert_eq!(Acorn::generate_bounds(13, &NumType::U128), (1_000_000_000_000, 9_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(14, &NumType::U128), (10_000_000_000_000, 99_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(15, &NumType::U128), (100_000_000_000_000, 999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(16, &NumType::U128), (1_000_000_000_000_000, 9_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(17, &NumType::U128), (10_000_000_000_000_000, 99_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(18, &NumType::U128), (100_000_000_000_000_000, 999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(19, &NumType::U128), (1_000_000_000_000_000_000, 9_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(21, &NumType::U128),
            (100_000_000_000_000_000_000, 999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(22, &NumType::U128),
            (1_000_000_000_000_000_000_000, 9_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(23, &NumType::U128),
            (10_000_000_000_000_000_000_000, 99_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(24, &NumType::U128),
            (100_000_000_000_000_000_000_000, 999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(25, &NumType::U128),
            (1_000_000_000_000_000_000_000_000, 9_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(26, &NumType::U128),
            (10_000_000_000_000_000_000_000_000, 99_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(27, &NumType::U128),
            (100_000_000_000_000_000_000_000_000, 999_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(28, &NumType::U128),
            (1_000_000_000_000_000_000_000_000_000, 9_999_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(29, &NumType::U128),
            (10_000_000_000_000_000_000_000_000_000, 99_999_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(30, &NumType::U128),
            (100_000_000_000_000_000_000_000_000_000, 999_999_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(31, &NumType::U128),
            (1_000_000_000_000_000_000_000_000_000_000, 9_999_999_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(32, &NumType::U128),
            (10_000_000_000_000_000_000_000_000_000_000, 99_999_999_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(33, &NumType::U128),
            (100_000_000_000_000_000_000_000_000_000_000, 999_999_999_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(34, &NumType::U128),
            (1_000_000_000_000_000_000_000_000_000_000_000, 9_999_999_999_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(35, &NumType::U128),
            (10_000_000_000_000_000_000_000_000_000_000_000, 99_999_999_999_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(36, &NumType::U128),
            (100_000_000_000_000_000_000_000_000_000_000_000, 999_999_999_999_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(37, &NumType::U128),
            (1_000_000_000_000_000_000_000_000_000_000_000_000, 9_999_999_999_999_999_999_999_999_999_999_999_999));
        assert_eq!(Acorn::generate_bounds(38, &NumType::U128),
            (10_000_000_000_000_000_000_000_000_000_000_000_000, 99_999_999_999_999_999_999_999_999_999_999_999_999));
    }
    #[test]
    fn new_range_from_zero() {
        let mut prng = Acorn::new(Order::new(45), Seed::new(1_000_000));
        assert_eq!(prng.generate_from_zero_range(9999), 7516);
        assert_eq!(prng.generate_from_zero_range(u128::MAX), 1196907755810977596096526034568560364);
    }
}
