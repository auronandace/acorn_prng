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
    fn generate_bounds(length: usize, num_type: &NumType) -> (u128, u128) {
        match length {
            1 => (0, 9),
            2 => (10, 99),
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
                y: alloc::vec![
                    1_000_001,
                    68_000_068,
                    2_346_002_346,
                    54_740_054_740,
                    971_635_971_635,
                    13_991_557_991_544,
                    170_230_622_230_452,
                    1_799_580_863_579_064,
                    16_871_070_596_053_725,
                    142_466_818_366_675_900,
                    1_096_994_501_423_404_430,
                    7_778_688_282_820_504_140,
                    51_209_697_861_901_652_255,
                    315_136_602_227_087_090_800,
                    1_823_290_341_456_718_168_200,
                    9_967_320_533_296_725_986_160,
                    51_705_475_266_476_766_053_205,
                    255_485_877_787_296_961_674_660,
                    1_206_461_089_551_124_541_241_450,
                    5_460_823_879_020_879_502_461_300,
                    23_754_583_873_740_825_835_706_655,
                    99_543_018_137_580_603_502_008_840,
                    402_696_755_192_939_714_167_217_580,
                    1_575_769_911_624_546_707_610_851_400,
                    5_974_794_248_243_072_933_024_478_225,
                    21_987_242_833_534_508_393_530_079_868,
                    78_646_676_289_181_126_176_857_593_374,
                    273_806_947_080_852_809_652_763_473_228,
                    928_987_856_167_179_175_607_590_355_595,
                    3_075_270_144_553_420_719_252_712_901_280,
                    9_943_373_467_389_393_658_917_105_047_472,
                    31_433_890_316_263_244_470_125_041_762_976,
                    97_248_598_165_939_412_579_449_347_954_207,
                    294_692_721_714_967_916_907_422_266_527_900,
                    875_410_732_153_287_047_283_813_203_509_350,
                    2_551_196_990_846_722_252_084_255_621_655_820,
                    7_299_258_057_144_788_665_685_509_139_737_485,
                    20_516_833_457_920_487_060_305_214_879_262_120,
                    56_691_250_344_253_977_403_474_935_850_592_700,
                    154_083_911_192_074_912_942_778_030_773_405_800,
                    412_174_462_438_800_392_121_931_232_318_860_515,
                    1_085_727_852_277_815_667_052_892_026_596_022_820, // ceiling passed after this one
                    159_266_291_722_594_628_210_605_662_748_036_738,
                    561_986_792_288_604_382_969_433_911_713_622_420,
                    904_174_045_811_170_833_414_601_003_987_414_337],
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
    fn bounds_testing() {
        assert_eq!(Acorn::generate_bounds(1, &NumType::U128), (0, 9));
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
        assert_eq!(prng.generate_from_zero_range(u128::MAX), 1_196_907_755_810_977_596_096_526_034_568_560_364);
    }
}
