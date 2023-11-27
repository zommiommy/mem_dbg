/*
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![doc = include_str!("../../README.md")]
#![deny(unconditional_recursion)]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

// Re-export mem_dbg_derive conditional to the "derive" feature.
//
// The reason re-exporting is not enabled by default is that disabling it would
// be annoying for crates that provide handwritten impls or data formats. They
// would need to disable default features and then explicitly re-enable std.
#[cfg(feature = "derive")]
extern crate mem_dbg_derive;
#[cfg(feature = "derive")]
pub use mem_dbg_derive::{MemDbg, MemSize};

mod impl_mem_dbg;
mod impl_mem_size;
pub mod utils;

bitflags::bitflags! {
    /// Flags for [`MemDbg`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SizeFlags: u32 {
        /// Follow references.
        ///
        /// By default [`MemSize::mem_size`] does not follow references
        /// and computes only the size of the reference itself.
        const FOLLOW_REFS = 1 << 0;
        /// Return capacity instead of size.
        ///
        /// Size does not include memory allocated but not
        /// used: for example, in the case of a vector this option
        /// makes [`MemSize::mem_size`] call [`Vec::len`] rather than [`Vec::capacity`].
        ///
        /// Capacity includes also memory allocated but not
        /// used: for example, in the case of a vector this function
        /// makes [`MemSize::mem_size`] call [`Vec::capacity`] rather than [`Vec::len`].
        const CAPACITY = 1 << 1;
    }
}

impl Default for SizeFlags {
    /// The default set of flags is the empty set.
    #[inline(always)]
    fn default() -> Self {
        Self::empty()
    }
}

/// A trait to compute recursively the overall size and capacity of a structure, as opposed to the
/// stack size returned by [`core::mem::size_of()`].
pub trait MemSize {
    /// Return the (recursively computed) overall
    /// memory size of the structure in bytes.
    fn mem_size(&self, flags: SizeFlags) -> usize;
}

bitflags::bitflags! {
    /// Flags for [`MemDbg`].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct DbgFlags: u32 {
        /// Follow references.
        const FOLLOW_REFS = 1 << 0;
        /// Print memory usage in human readable format.
        const HUMANIZE = 1 << 1;
        /// Print memory usage as a percentage.
        const PERCENTAGE = 1 << 2;
        /// Print the type name.
        const TYPE_NAME = 1 << 3;
        /// Display capacity instead of size.
        const CAPACITY = 1 << 4;
        /// Add an _ every 3 digits.
        const SEPARATOR = 1 << 5;
    }
}

impl DbgFlags {
    pub fn to_size_flags(&self) -> SizeFlags {
        let mut flags = SizeFlags::empty();
        if self.contains(Self::all()) {
            flags |= SizeFlags::FOLLOW_REFS;
        }
        if self.contains(Self::all()) {
            flags |= SizeFlags::CAPACITY;
        }
        flags
    }
}

impl Default for DbgFlags {
    /// The default set of flags contains [`DbgFlags::TYPE_NAME`].
    #[inline(always)]
    fn default() -> Self {
        Self::TYPE_NAME | Self::SEPARATOR
    }
}

/// A trait providing methods to display recursively the content
/// and size of a structure.
///
/// You can derive this trait with `#[derive(MemDbg)]` if all the fields of your structure
/// implement [`MemDbg`]. Note that you will also need to derive [`MemSize`].
pub trait MemDbg: MemDbgImpl {
    /// Print debug infos about the structure memory usage, expanding
    /// all levels of nested structures.
    #[cfg(feature = "std")]
    #[inline(always)]
    fn mem_dbg(&self, flags: DbgFlags) -> core::fmt::Result {
        self.mem_dbg_depth(
            self.mem_size(flags.to_size_flags()),
            0,
            usize::MAX,
            false,
            flags,
        )
    }

    /// Print debug infos about the structure memory usage, expanding
    /// all levels of nested structures.
    #[inline(always)]
    fn mem_dbg_on(&self, writer: &mut impl core::fmt::Write, flags: DbgFlags) -> core::fmt::Result {
        self.mem_dbg_depth_on(
            writer,
            self.mem_size(flags.to_size_flags()),
            0,
            usize::MAX,
            Some("⏺"),
            false,
            flags,
        )
    }

    /// Write to stdout debug infos about the structure memory usage, but expanding only
    /// up to `max_depth` levels of nested structures.
    #[cfg(feature = "std")]
    #[inline(always)]
    fn mem_dbg_depth(
        &self,
        total_size: usize,
        depth: usize,
        max_depth: usize,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        struct Wrapper(std::io::Stdout);
        impl core::fmt::Write for Wrapper {
            #[inline(always)]
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                use std::io::Write;
                self.0
                    .lock()
                    .write(s.as_bytes())
                    .map_err(|_| core::fmt::Error)
                    .map(|_| ())
            }
        }
        self.mem_dbg_depth_on(
            &mut Wrapper(std::io::stdout()),
            total_size,
            depth,
            max_depth,
            Some("⏺"),
            is_last,
            flags,
        )
    }

    /// Write the data on `writer` debug infos about the structure memory usage, but expanding only
    /// up to `max_depth` levels of nested structures.
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn mem_dbg_depth_on(
        &self,
        writer: &mut impl core::fmt::Write,
        total_size: usize,
        depth: usize,
        max_depth: usize,
        field_name: Option<&str>,
        is_last: bool,
        flags: DbgFlags,
    ) -> core::fmt::Result {
        if depth > max_depth {
            return Ok(());
        }
        let mut real_size = self.mem_size(flags.to_size_flags());
        if flags.contains(DbgFlags::PERCENTAGE) {
            writer.write_fmt(format_args!(
                "{:>6.2}% ",
                100.0 * real_size as f64 / total_size as f64
            ))?;
        } else if flags.contains(DbgFlags::HUMANIZE) {
            let (value, uom) = crate::utils::humanize_float(real_size as f64);
            if uom == " B" {
                writer.write_fmt(format_args!("{:>6} B ", real_size))?;
            } else {
                let mut precision = 4;
                let a = value.abs();
                if a >= 100.0 {
                    precision = 1;
                } else if a >= 10.0 {
                    precision = 2;
                } else if a >= 1.0 {
                    precision = 3;
                }
                writer.write_fmt(format_args!("{0:>4.1$} {2}", value, precision, uom))?;
            }
        } else if flags.contains(DbgFlags::SEPARATOR) {
            let mut align = crate::utils::n_of_digits(total_size);
            align += align / 3;
            let mut digits = crate::utils::n_of_digits(real_size);
            let digit_align = digits + digits / 3;
            for _ in digit_align..align {
                writer.write_char(' ')?;
            }

            let first_digits = digits % 3;
            let mut multiplier = 10_usize.pow((digits - first_digits) as u32);
            if first_digits != 0 {
                writer.write_fmt(format_args!("{}", real_size / multiplier))?;
            } else {
                multiplier /= 1000;
                digits -= 3;
                writer.write_fmt(format_args!(" {}", real_size / multiplier))?;
            }

            while digits >= 3 {
                real_size %= multiplier;
                multiplier /= 1000;
                writer.write_fmt(format_args!("_{:03}", real_size / multiplier))?;
                digits -= 3;
            }

            writer.write_str(" B ")?;
        } else {
            let align = crate::utils::n_of_digits(total_size);
            writer.write_fmt(format_args!("{:>align$} B ", real_size, align = align))?;
        }

        let indent = "│".repeat(depth.saturating_sub(1));
        writer.write_str(&indent)?;
        if depth > 0 {
            if is_last {
                writer.write_char('╰')?;
            } else {
                writer.write_char('├')?;
            }
            writer.write_char('╴')?;
        }

        if let Some(field_name) = field_name {
            writer.write_fmt(format_args!("{:}", field_name))?;
        }

        if flags.contains(DbgFlags::TYPE_NAME) {
            writer.write_str(" : ")?;
            writer.write_fmt(format_args!("{:}", core::any::type_name::<Self>()))?;
        }

        writer.write_char('\n')?;

        self._mem_dbg_rec_on(writer, total_size, depth + 1, max_depth, false, flags)
    }
}

/// Implement [`MemDbg`] for all types that implement [`MemDbgImpl`].
/// This is done so that no one can change the implementation of [`MemDbg`],
/// this ensures consistency in printing.
impl<T: MemDbgImpl> MemDbg for T {}

/// Inner trait used to implement [`MemDbg`].
///
/// This trait should not be implemented by users, but they should use the
/// [`MemDbg`] derive macro instead.
///
/// The default no-op implementation is used by primitive types.
pub trait MemDbgImpl: MemSize {
    #[inline(always)]
    /// Composite structs should implement this to print their children.
    fn _mem_dbg_rec_on(
        &self,
        _writer: &mut impl core::fmt::Write,
        _total_size: usize,
        _depth: usize,
        _max_depth: usize,
        _is_last: bool,
        _flags: DbgFlags,
    ) -> core::fmt::Result {
        Ok(())
    }
}
