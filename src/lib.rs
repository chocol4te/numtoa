//! The standard library provides a convenient method of converting numbers into strings, but these strings are
//! heap-allocated. If you have an application which needs to convert large volumes of numbers into strings, but don't
//! want to pay the price of heap allocation, this crate provides an efficient `no_std`-compatible method of heaplessly converting numbers
//! into their string representations, storing the representation within a reusable byte array.
//!
//! In addition to supporting the standard base 10 conversion, this implementation allows you to select the base of
//! your choice. Therefore, if you want a binary representation, set the base to 2. If you want hexadecimal, set the
//! base to 16.
//!
//! ## Base 10 Example
//! ```
//! use numtoa::NumToA;
//! use std::io::{self, Write};
//!
//! let stdout = io::stdout();
//! let mut stdout = stdout.lock();
//! let mut buffer = [0u8; 20];
//!
//! let number: u32 = 162392;
//! let mut bytes_written = number.numtoa(10, &mut buffer);
//! let _ = stdout.write(&buffer[0..bytes_written]);
//! let _ = stdout.write(b"\n");
//! assert_eq!(&buffer[0..bytes_written], "162392".as_bytes());
//!
//! let other_number: i32 = -6235;
//! bytes_written = other_number.numtoa(10, &mut buffer);
//! let _ = stdout.write(&buffer[0..bytes_written]);
//! let _ = stdout.write(b"\n");
//! assert_eq!(&buffer[0..bytes_written], "-6235".as_bytes());
//!
//! let large_num: u64 = 35320842;
//! bytes_written = large_num.numtoa(10, &mut buffer);
//! let _ = stdout.write(&buffer[0..bytes_written]);
//! let _ = stdout.write(b"\n");
//! assert_eq!(&buffer[0..bytes_written], "35320842".as_bytes());
//!
//! let max_u64: u64 = 18446744073709551615;
//! bytes_written = max_u64.numtoa(10, &mut buffer);
//! let _ = stdout.write(&buffer[0..bytes_written]);
//! let _ = stdout.write(b"\n");
//! assert_eq!(&buffer[0..bytes_written], "18446744073709551615".as_bytes());
//! ```

#![no_std]
use core::ptr::swap;

/// Converts a number into a string representation, storing the conversion into a mutable byte slice.
pub trait NumToA<T> {
    /// Given a base for encoding and a mutable byte slice, write the number into the byte slice and return the
    /// amount of bytes that were written.
    ///
    /// # Panics
    /// If the supplied buffer is smaller than the number of bytes needed to write the integer, this will panic.
    fn numtoa(self, base: T, string: &mut [u8]) -> usize;
}

// A lookup table to prevent the need for conditional branching
// The value of the remainder of each step will be used as the index
const LOOKUP: &'static [u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// Because the integer to string conversion writes the representation in reverse, this will correct it.
fn reverse(string: &mut [u8], length: usize) {
    let mut start = 0isize;
    let mut end   = length as isize - 1;
    while start < end {
        unsafe {
            let x = string.as_mut_ptr().offset(start);
            let y = string.as_mut_ptr().offset(end);
            swap(x, y);
        }
        start += 1;
        end -= 1;
    }
}

macro_rules! base_10 {
    ($number:ident, $index:ident, $string:ident) => {
        // Decode four characters at the same time
        while $number > 9999 {
            let rem = $number % 10000;
            $string[$index+3] = LOOKUP[(rem / 1000) as usize];
            $string[$index+2] = LOOKUP[(rem % 1000 / 100) as usize];
            $string[$index+1] = LOOKUP[(rem % 100 / 10) as usize];
            $string[$index]   = LOOKUP[(rem % 10) as usize];
            $index += 4;
            $number /= 10000;
        }

        if $number > 999 {
            let rem = $number % 1000;
            $string[$index+3] = LOOKUP[($number / 1000) as usize];
            $string[$index+2] = LOOKUP[(rem / 100) as usize];
            $string[$index+1] = LOOKUP[(rem % 100 / 10) as usize];
            $string[$index]   = LOOKUP[(rem % 10) as usize];
            $index += 4;
        } else if $number > 99 {
            let rem = $number % 100;
            $string[$index+2] = LOOKUP[($number / 100) as usize];
            $string[$index+1] = LOOKUP[(rem / 10) as usize];
            $string[$index]   = LOOKUP[(rem % 10) as usize];
            $index += 3;
        } else if $number > 9 {
            $string[$index+1] = LOOKUP[($number / 10) as usize];
            $string[$index]   = LOOKUP[($number % 10) as usize];
            $index += 2;
        } else {
            $string[$index] = LOOKUP[$number as usize];
            $index += 1;
        }
    }
}

macro_rules! impl_unsized_numtoa_for {
    ($t:ty) => {
        impl NumToA<$t> for $t {
            fn numtoa(mut self, base: $t, string: &mut [u8]) -> usize {
                if self == 0 {
                    string[0] = b'0';
                    return 1;
                }

                let mut index = 0;

                if base == 10 {
                    base_10!(self, index, string);
                } else {
                    while self != 0 {
                        let rem = self % base;
                        string[index] = LOOKUP[rem as usize];
                        index += 1;
                        self /= base;
                    }
                }

                reverse(string, index);
                index
            }
        }
    }
}

macro_rules! impl_sized_numtoa_for {
    ($t:ty) => {
        impl NumToA<$t> for $t {
            fn numtoa(mut self, base: $t, string: &mut [u8]) -> usize {
                let mut index = 0;
                let mut is_negative = false;

                if self < 0 {
                    is_negative = true;
                    self = self.abs();
                } else if self == 0 {
                    string[0] = b'0';
                    return 1;
                }

                if base == 10 {
                    base_10!(self, index, string);
                } else {
                    while self != 0 {
                        let rem = self % base;
                        string[index] = LOOKUP[rem as usize];
                        index += 1;
                        self /= base;
                    }
                }

                if is_negative {
                    string[index] = b'-';
                    index += 1;
                }

                reverse(string, index);
                index
            }
        }

    }
}

impl_sized_numtoa_for!(i16);
impl_sized_numtoa_for!(i32);
impl_sized_numtoa_for!(i64);
impl_sized_numtoa_for!(isize);
impl_unsized_numtoa_for!(u16);
impl_unsized_numtoa_for!(u32);
impl_unsized_numtoa_for!(u64);
impl_unsized_numtoa_for!(usize);

impl NumToA<i8> for i8 {
    fn numtoa(mut self, base: i8, string: &mut [u8]) -> usize {
        let mut index = 0;
        let mut is_negative = false;

        if self < 0 {
            is_negative = true;
            self = self.abs();
        } else if self == 0 {
            string[0] = b'0';
            return 1;
        }

        while self != 0 {
            let rem = self % base;
            string[index] = LOOKUP[rem as usize];
            index += 1;
            self /= base;
        }

        if is_negative {
            string[index] = b'-';
            index += 1;
        }

        reverse(string, index);
        index
    }
}

impl NumToA<u8> for u8 {
    fn numtoa(mut self, base: u8, string: &mut [u8]) -> usize {
        if self == 0 {
            string[0] = b'0';
            return 1;
        }

        let mut index = 0;
        while self != 0 {
            let rem = self % base;
            string[index] = LOOKUP[rem as usize];
            index += 1;
            self /= base;
        }

        reverse(string, index);
        index
    }
}
