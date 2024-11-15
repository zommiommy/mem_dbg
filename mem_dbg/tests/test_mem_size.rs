/*
 * SPDX-FileCopyrightText: 2023 Tommaso Fontana
 * SPDX-FileCopyrightText: 2023 Inria
 * SPDX-FileCopyrightText: 2023 Sebastiano Vigna
 *
 * SPDX-License-Identifier: Apache-2.0 OR LGPL-2.1-or-later
 */

#![cfg_attr(feature = "offset_of_enum", feature(offset_of_enum, offset_of_nested))]

use core::marker::PhantomData;
use core::mem::size_of;
use mem_dbg::*;
use std::sync::atomic::AtomicU64;

#[derive(MemSize, MemDbg)]
union SingletonUnion<A: Copy> {
    a: A,
}

#[allow(dead_code)]
#[derive(MemSize, MemDbg)]
enum TestEnum {
    Unit,
    Unit2(),
    Unit3 {},
    Union(SingletonUnion<u8>),
    Unnamed(usize, u8),
    Named {
        first: usize,
        second: PhantomData<u8>,
    },
}

#[test]
fn test_vec_capacity() {
    let mut v = vec![Vec::with_capacity(10)];
    v[0].push(1);
    v[0].push(2);
    // We consider the capacity of the inner vector
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY) - v.mem_size(SizeFlags::default()),
        8 * std::mem::size_of::<i32>()
    );
}

#[test]
fn test_vec_copy_or_not() {
    #[derive(MemDbg, MemSize, Clone)]
    struct NewType(usize);

    assert_eq!(
        vec![NewType(1_usize); 10].mem_size(SizeFlags::default()),
        vec![1_usize; 10].mem_size(SizeFlags::default())
    );
}

#[test]
fn test_boxed_slice_copy_or_not() {
    #[derive(MemDbg, MemSize, Clone)]
    struct NewType(usize);

    assert_eq!(
        vec![NewType(1_usize); 10]
            .into_boxed_slice()
            .mem_size(SizeFlags::FOLLOW_REFS),
        vec![1_usize; 10]
            .into_boxed_slice()
            .mem_size(SizeFlags::FOLLOW_REFS)
    );
}

#[test]
fn test_slice_copy_or_not() {
    #[derive(MemDbg, MemSize, Clone)]
    struct NewType(usize);

    assert_eq!(
        vec![NewType(1_usize); 10]
            .into_boxed_slice()
            .as_ref()
            .mem_size(SizeFlags::FOLLOW_REFS),
        vec![1_usize; 10]
            .into_boxed_slice()
            .as_ref()
            .mem_size(SizeFlags::FOLLOW_REFS)
    );
}

#[test]
fn test_array_copy_or_not() {
    #[derive(MemDbg, MemSize, Clone, Copy)]
    struct NewType(usize);

    assert_eq!(
        [NewType(1_usize); 10]
            .as_ref()
            .mem_size(SizeFlags::FOLLOW_REFS),
        [1_usize; 10].as_ref().mem_size(SizeFlags::FOLLOW_REFS)
    );
}

#[test]
fn test_empty_struct() {
    #[derive(MemSize, Clone, Copy)]
    #[copy_type]
    struct Data {}
    let v = Data {};
    assert_eq!(v.mem_size(SizeFlags::default()), 0);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), 0);
}

#[test]
fn test_struct() {
    #[derive(MemSize)]
    struct Data {
        a: u64,
        b: Vec<usize>,
    }
    let mut v = Data {
        a: 10,
        b: Vec::with_capacity(10),
    };
    v.b.push(1);
    v.b.push(2);
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        8 + v.b.mem_size(SizeFlags::default())
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        8 + v.b.mem_size(SizeFlags::CAPACITY)
    );
}

#[test]
fn test_empty_tuple_struct() {
    #[derive(MemSize)]
    struct Data();
    let v = Data();
    assert_eq!(v.mem_size(SizeFlags::default()), 0);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), 0);
}

#[test]
fn test_tuple_struct() {
    #[derive(MemSize)]
    struct Data(u64, Vec<usize>);
    let mut v = Data(10, Vec::with_capacity(10));
    v.1.push(1);
    v.1.push(2);
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        8 + v.1.mem_size(SizeFlags::default())
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        8 + v.1.mem_size(SizeFlags::CAPACITY),
    );
}

#[test]
fn test_padding() {
    assert_eq!((0_u8, 0_u64).mem_size(SizeFlags::default()), 16);
    #[derive(MemSize)]
    struct TuplePadded((u8, u64));
    let v = TuplePadded((0, 0));
    assert_eq!(v.mem_size(SizeFlags::default()), 16);

    #[derive(MemSize)]
    struct StructPadded(u8, u64);
    let v = StructPadded(0, 0);
    assert_eq!(v.mem_size(SizeFlags::default()), 16);

    #[derive(MemSize)]
    struct StructStructPadded(StructPadded);
    let v = StructStructPadded(StructPadded(0, 0));
    assert_eq!(v.mem_size(SizeFlags::default()), 16);
}

#[test]
fn test_option() {
    let v = Some(1_usize);
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        2 * core::mem::size_of::<usize>(),
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        v.mem_size(SizeFlags::default())
    );
    let v = Some(Some(1_usize));
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        2 * core::mem::size_of::<usize>(),
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        v.mem_size(SizeFlags::default())
    );
    let v = Some(Some(Some(1_usize)));
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        2 * core::mem::size_of::<usize>(),
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        v.mem_size(SizeFlags::default())
    );
}

#[test]
fn test_enum() {
    #[derive(MemSize)]
    #[repr(u8)]
    enum Data {
        A,
        B(u64),
        C(u64, Vec<usize>),
    }

    let enum_size = core::mem::size_of::<Data>();

    let v = Data::A;
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);
    let v = Data::B(1000);
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);
    let d = vec![1, 2, 3, 4, 5];
    let len = d.len();
    let capacity = d.capacity();
    let v = Data::C(1000, d);
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        enum_size + core::mem::size_of::<usize>() * len
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        enum_size + core::mem::size_of::<usize>() * capacity
    );
}

#[test]
/// <https://github.com/rust-lang/rfcs/issues/1230>
fn test_exotic() {
    // A reference cannot be null, so the compiler should use null as Option's
    // None variant
    let v: Option<&u8> = None;
    assert_eq!(core::mem::size_of::<usize>(), core::mem::size_of::<&u8>());
    assert_eq!(
        core::mem::size_of::<usize>(),
        core::mem::size_of::<Option<&u8>>()
    );
    assert_eq!(
        v.mem_size(SizeFlags::default()),
        core::mem::size_of::<usize>()
    );
    assert_eq!(
        v.mem_size(SizeFlags::CAPACITY),
        core::mem::size_of::<usize>()
    );

    #[derive(MemSize)]
    enum Data1 {
        A,
        B,
    }
    #[derive(MemSize)]
    enum Data2 {
        A,
        B(Data1),
    }

    // nested enums can be flattened IFF they don't have a repr attribute
    assert_eq!(core::mem::size_of::<Data1>(), core::mem::size_of::<Data2>(),);

    let enum_size = core::mem::size_of::<Data1>();
    let v = Data1::A;
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);
    let v = Data1::B;
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);

    let enum_size = core::mem::size_of::<Data2>();
    let v = Data2::A;
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);
    let v = Data2::B(Data1::A);
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);
    let v = Data2::B(Data1::B);
    assert_eq!(v.mem_size(SizeFlags::default()), enum_size);
    assert_eq!(v.mem_size(SizeFlags::CAPACITY), enum_size);
}

#[test]
fn test_tuple() {
    assert_eq!((8, 4).mem_size(SizeFlags::default()), 8);
    assert_eq!((8, 4).mem_size(SizeFlags::CAPACITY), 8);
}

#[test]
fn test_atomic() {
    assert_eq!(AtomicU64::new(0).mem_size(SizeFlags::default()), 8);
}

#[test]
fn test_unit() {
    assert_eq!(().mem_size(SizeFlags::default()), 0);
}

#[test]
#[cfg(feature = "std")]
fn test_phantom() {
    struct Dummy();
    #[derive(MemSize, MemDbg)]
    struct Example<A>(PhantomData<A>);

    Example::<Dummy>(PhantomData)
        .mem_dbg(DbgFlags::default())
        .unwrap();
}

#[test]
#[cfg(feature = "std")]
fn test_vec_strings() {
    let data = vec![String::new(), String::new()];
    data.mem_dbg(DbgFlags::default()).unwrap();
}

#[test]
#[cfg(feature = "std")]
fn test_array_u8() {
    let data = [0_u8; 10];
    assert_eq!(data.mem_size(SizeFlags::default()), 10);
    data.mem_dbg(DbgFlags::default()).unwrap();
}

#[test]
#[cfg(feature = "std")]
fn test_array_empty_struct() {
    #[derive(MemSize, MemDbg, Clone, Copy)]
    struct Dummy;
    let data = [Dummy; 10];
    assert_eq!(data.mem_size(SizeFlags::default()), 0);
    data.mem_dbg(DbgFlags::default()).unwrap();
}

#[test]
fn test_slice_u8() {
    let data = [0_u8; 10].as_slice();
    assert_eq!((*data).mem_size(SizeFlags::default()), 10);
    // Autodereferentiation
    assert_eq!(data.mem_size(SizeFlags::default()), 10);
    assert_eq!(
        <&[u8] as MemSize>::mem_size(&data, SizeFlags::default()),
        16
    );
    assert_eq!(
        <&[u8] as MemSize>::mem_size(&data, SizeFlags::default() | SizeFlags::FOLLOW_REFS),
        26
    );
}

#[test]
fn test_slice_empty_struct() {
    #[derive(MemSize, MemDbg, Clone, Copy)]
    struct Dummy;
    let data = [Dummy; 10].as_slice();
    assert_eq!((*data).mem_size(SizeFlags::default()), 0);
    // Autodereferentiation
    assert_eq!(data.mem_size(SizeFlags::default()), 0);
    assert_eq!(
        <&[Dummy] as MemSize>::mem_size(&data, SizeFlags::default()),
        16
    );
    assert_eq!(
        <&[Dummy] as MemSize>::mem_size(&data, SizeFlags::default() | SizeFlags::FOLLOW_REFS),
        16
    );
}

#[test]
#[cfg(feature = "std")]
fn test_indirect_call() {
    #[derive(MemSize, MemDbg)]
    struct Dummy<T>(Vec<T>);

    fn test<T>(data: Vec<T>)
    where
        // this is needed because the type system is not smart enough to infer it
        Vec<T>: MemSize + MemDbgImpl,
    {
        let data = Dummy(data);
        data.mem_dbg(DbgFlags::default()).unwrap();
    }

    test(vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_vec_slice_i64() {
    let mut data: Vec<i64> = vec![1, 2, 3, 4, 5];

    // A mutable slice should have the same size as a non mutable one
    let non_mutable_slice = data.as_slice();
    let size_of_non_mutable_slice = core::mem::size_of_val(non_mutable_slice);
    let non_mutable_slice_shallow_size =
        <&[i64] as MemSize>::mem_size(&non_mutable_slice, SizeFlags::default());
    let non_mutable_slice_deep_size = <&[i64] as MemSize>::mem_size(
        &non_mutable_slice,
        SizeFlags::default() | SizeFlags::FOLLOW_REFS,
    );
    let mutable_slice = data.as_mut_slice();
    let size_of_mutable_slice = core::mem::size_of_val(mutable_slice);

    let mutable_slice_shallow_size =
        <&mut [i64] as MemSize>::mem_size(&mutable_slice, SizeFlags::default());
    let mutable_slice_deep_size = <&mut [i64] as MemSize>::mem_size(
        &mutable_slice,
        SizeFlags::default() | SizeFlags::FOLLOW_REFS,
    );

    assert_eq!(
        mutable_slice_shallow_size, non_mutable_slice_shallow_size,
        "Expected mutable slice shallow size to be identical to non mutable slice shallow size"
    );

    assert_eq!(
        mutable_slice_deep_size, non_mutable_slice_deep_size,
        "Expected mutable slice deep size to be identical to non mutable slice deep size"
    );

    assert_eq!(non_mutable_slice_shallow_size, size_of::<&[i64]>());

    assert_eq!(mutable_slice_shallow_size, size_of::<&mut [i64]>());

    assert_eq!(
        non_mutable_slice_deep_size,
        size_of::<&[i64]>() + size_of_non_mutable_slice
    );

    assert_eq!(
        mutable_slice_deep_size,
        size_of::<&mut [i64]>() + size_of_mutable_slice
    );
}

#[test]
fn test_vec_slice_i32() {
    let mut data: Vec<i32> = vec![1, 2, 3, 4, 5];

    // A mutable slice should have the same size as a non mutable one
    let non_mutable_slice = data.as_slice();
    let size_of_non_mutable_slice = core::mem::size_of_val(non_mutable_slice);
    let non_mutable_slice_shallow_size =
        <&[i32] as MemSize>::mem_size(&non_mutable_slice, SizeFlags::default());
    let non_mutable_slice_deep_size = <&[i32] as MemSize>::mem_size(
        &non_mutable_slice,
        SizeFlags::default() | SizeFlags::FOLLOW_REFS,
    );
    let mutable_slice = data.as_mut_slice();
    let size_of_mutable_slice = core::mem::size_of_val(mutable_slice);
    let mutable_slice_shallow_size =
        <&mut [i32] as MemSize>::mem_size(&mutable_slice, SizeFlags::default());
    let mutable_slice_deep_size = <&mut [i32] as MemSize>::mem_size(
        &mutable_slice,
        SizeFlags::default() | SizeFlags::FOLLOW_REFS,
    );

    assert_eq!(
        mutable_slice_shallow_size, non_mutable_slice_shallow_size,
        "Expected mutable slice shallow size to be identical to non mutable slice shallow size"
    );

    assert_eq!(
        mutable_slice_deep_size, non_mutable_slice_deep_size,
        "Expected mutable slice deep size to be identical to non mutable slice deep size"
    );

    assert_eq!(non_mutable_slice_shallow_size, size_of::<&[i64]>());

    assert_eq!(mutable_slice_shallow_size, size_of::<&mut [i64]>());

    assert_eq!(
        non_mutable_slice_deep_size,
        size_of::<&[i64]>() + size_of_non_mutable_slice
    );

    assert_eq!(
        mutable_slice_deep_size,
        size_of::<&mut [i64]>() + size_of_mutable_slice
    );
}

#[test]
fn test_array_slice_i64() {
    let mut data: [i64; 5] = [1, 2, 3, 4, 5];

    // A mutable slice should have the same size as a non mutable one
    let non_mutable_slice = data.as_slice();
    let size_of_non_mutable_slice = core::mem::size_of_val(non_mutable_slice);
    let non_mutable_slice_shallow_size =
        <&[i64] as MemSize>::mem_size(&non_mutable_slice, SizeFlags::default());
    let non_mutable_slice_deep_size = <&[i64] as MemSize>::mem_size(
        &non_mutable_slice,
        SizeFlags::default() | SizeFlags::FOLLOW_REFS,
    );
    let mutable_slice = data.as_mut_slice();
    let size_of_mutable_slice = core::mem::size_of_val(mutable_slice);
    let mutable_slice_shallow_size =
        <&mut [i64] as MemSize>::mem_size(&mutable_slice, SizeFlags::default());
    let mutable_slice_deep_size = <&mut [i64] as MemSize>::mem_size(
        &mutable_slice,
        SizeFlags::default() | SizeFlags::FOLLOW_REFS,
    );

    assert_eq!(
        mutable_slice_shallow_size, non_mutable_slice_shallow_size,
        "Expected mutable slice shallow size to be identical to non mutable slice shallow size"
    );

    assert_eq!(
        mutable_slice_deep_size, non_mutable_slice_deep_size,
        "Expected mutable slice deep size to be identical to non mutable slice deep size"
    );

    assert_eq!(
        non_mutable_slice_deep_size,
        core::mem::size_of::<&[i64]>() + size_of_non_mutable_slice
    );

    assert_eq!(
        mutable_slice_deep_size,
        core::mem::size_of::<&mut [i64]>() + size_of_mutable_slice
    );
}

#[test]
#[should_panic]
fn test_array_slice_i64_without_specifications() {
    let mut data: [i64; 5] = [1, 2, 3, 4, 5];

    // A mutable slice should have the same size as a non mutable one
    let non_mutable_slice_shallow_size = data.as_slice().mem_size(SizeFlags::default());
    let non_mutable_slice_deep_size = data
        .as_slice()
        .mem_size(SizeFlags::default() | SizeFlags::FOLLOW_REFS);
    let mutable_slice_shallow_size = data
        .as_mut_slice()
        .mem_size(SizeFlags::default() | SizeFlags::FOLLOW_REFS);
    let mutable_slice_deep_size = data
        .as_mut_slice()
        .mem_size(SizeFlags::default() | SizeFlags::FOLLOW_REFS);

    assert_eq!(
        mutable_slice_shallow_size, non_mutable_slice_shallow_size,
        "Expected mutable slice shallow size to be identical to non mutable slice shallow size"
    );

    assert_eq!(
        mutable_slice_deep_size, non_mutable_slice_deep_size,
        "Expected mutable slice deep size to be identical to non mutable slice deep size"
    );
}

#[test]
fn test_array_slice_i32() {
    let mut data: [i32; 5] = [1, 2, 3, 4, 5];

    // A mutable slice should have the same size as a non mutable one
    let non_mutable_slice = data.as_slice();
    let size_of_non_mutable_slice = core::mem::size_of_val(non_mutable_slice);
    let non_mutable_slice_shallow_size =
        <&[i32] as MemSize>::mem_size(&non_mutable_slice, SizeFlags::default());
    let non_mutable_slice_deep_size = <&[i32] as MemSize>::mem_size(
        &non_mutable_slice,
        SizeFlags::default() | SizeFlags::FOLLOW_REFS,
    );
    let mutable_slice = data.as_mut_slice();
    let size_of_mutable_slice = core::mem::size_of_val(mutable_slice);
    let mutable_slice_shallow_size =
        <&mut [i32] as MemSize>::mem_size(&mutable_slice, SizeFlags::default());
    let mutable_slice_deep_size = <&mut [i32] as MemSize>::mem_size(
        &mutable_slice,
        SizeFlags::default() | SizeFlags::FOLLOW_REFS,
    );

    assert_eq!(
        mutable_slice_shallow_size, non_mutable_slice_shallow_size,
        "Expected mutable slice shallow size to be identical to non mutable slice shallow size"
    );

    assert_eq!(
        mutable_slice_deep_size, non_mutable_slice_deep_size,
        "Expected mutable slice deep size to be identical to non mutable slice deep size"
    );

    assert_eq!(
        non_mutable_slice_deep_size,
        core::mem::size_of::<&[i64]>() + size_of_non_mutable_slice
    );

    assert_eq!(
        mutable_slice_deep_size,
        core::mem::size_of::<&mut [i64]>() + size_of_mutable_slice
    );
}

#[derive(MemSize)]
struct MutableSliceWrapper<'a> {
    data: &'a mut [i64],
}

#[derive(MemSize)]
struct NonMutableSliceWrapper<'a> {
    data: &'a [i64],
}

#[test]
fn test_compare_structs() {
    let mut data: [i64; 5] = [1, 2, 3, 4, 5];
    let mutable_slice = MutableSliceWrapper {
        data: data.as_mut_slice(),
    };
    let mutable_slice_shallow_size =
        <MutableSliceWrapper as MemSize>::mem_size(&mutable_slice, SizeFlags::default());
    let mutable_slice_deep_size = <MutableSliceWrapper as MemSize>::mem_size(
        &mutable_slice,
        SizeFlags::default() | SizeFlags::FOLLOW_REFS,
    );
    let non_mutable_slice = NonMutableSliceWrapper {
        data: data.as_slice(),
    };
    let non_mutable_slice_shallow_size =
        <NonMutableSliceWrapper as MemSize>::mem_size(&non_mutable_slice, SizeFlags::default());
    let non_mutable_slice_deep_size = <NonMutableSliceWrapper as MemSize>::mem_size(
        &non_mutable_slice,
        SizeFlags::default() | SizeFlags::FOLLOW_REFS,
    );

    assert_eq!(
        mutable_slice_shallow_size, non_mutable_slice_shallow_size,
        "Expected mutable slice shallow size to be identical to non mutable slice shallow size"
    );

    assert_eq!(
        mutable_slice_deep_size, non_mutable_slice_deep_size,
        "Expected mutable slice deep size to be identical to non mutable slice deep size"
    );
}

/// Macro to generate test functions given a type T and the expected size.
macro_rules! test_size {
    ($(($object:ty, $shallow_size:expr, $deep_size:expr)),*) => {
        $(
            paste::paste! {
                #[test]
                fn [<test_ $object:lower _size>]() {
                    // First, we check that the shallow size of the object is the expected one,
                    // meaning that we are not following any reference
                    let mut data: $object = Default::default();
                    let shallow_object_size = <$object as MemSize>::mem_size(&data, SizeFlags::default());
                    assert_eq!(
                        shallow_object_size,
                        $shallow_size,
                    );

                    // We check that any pointer is always the size of a usize,
                    // which is the size of a pointer on the current architecture
                    let reference = &data;
                    let reference_size = <&'_ $object as MemSize>::mem_size(&reference, SizeFlags::default());

                    assert_eq!(
                        reference_size,
                        core::mem::size_of::<usize>()
                    );

                    // Next, we check that the deep size of the object is the expected one,
                    // meaning that we are following all references
                    let deep_object_size = <$object as MemSize>::mem_size(&data, SizeFlags::default() | SizeFlags::FOLLOW_REFS);
                    assert_eq!(
                        deep_object_size,
                        $deep_size
                    );

                    // We now  check that the deep size of the reference to this object is equal to the size of a pointer
                    // plus the deep size of the object
                    let deep_reference_size = <&'_ $object as MemSize>::mem_size(&reference, SizeFlags::default() | SizeFlags::FOLLOW_REFS);
                    assert_eq!(
                        deep_reference_size,
                        core::mem::size_of::<usize>() + deep_object_size
                    );

                    let mutable_reference = &mut data;
                    let mutable_reference_size = <&'_ mut $object as MemSize>::mem_size(&mutable_reference, SizeFlags::default());
                    // We check that the mutable reference has the same size as the reference
                    assert_eq!(
                        mutable_reference_size,
                        core::mem::size_of::<usize>(),
                    );

                    // We check that the deep size of the mutable reference is the same as the deep size of the reference
                    // i.e. the size of a pointer plus the deep size of the object
                    let deep_mutable_reference_size = <&'_ mut $object as MemSize>::mem_size(&mutable_reference, SizeFlags::default() | SizeFlags::FOLLOW_REFS);
                    assert_eq!(
                        deep_mutable_reference_size,
                        deep_reference_size,
                    );
                }
            }
        )*
    };
}

#[derive(MemSize)]
enum TestEnum2 {
    A,
    _B(u64),
    _C(u64, Vec<usize>),
}

impl Default for TestEnum2 {
    fn default() -> Self {
        TestEnum2::A
    }
}

#[repr(u8)]
#[derive(MemSize)]
enum TestEnumReprU8 {
    A,
    _B(u64),
    _C(u64, Vec<usize>),
}

impl Default for TestEnumReprU8 {
    fn default() -> Self {
        TestEnumReprU8::A
    }
}

#[derive(MemSize, MemDbg)]
union TestUnion {
    a: u64,
}

impl Default for TestUnion {
    fn default() -> Self {
        TestUnion { a: 0 }
    }
}

test_size!(
    (u8, 1, 1),
    (u16, 2, 2),
    (u32, 4, 4),
    (u64, 8, 8),
    (u128, 16, 16),
    (i8, 1, 1),
    (i16, 2, 2),
    (i32, 4, 4),
    (i64, 8, 8),
    (i128, 16, 16),
    (f32, 4, 4),
    (f64, 8, 8),
    (bool, 1, 1),
    (char, 4, 4),
    (TestEnum2, 32, 32),
    (TestEnumReprU8, 40, 40),
    (TestUnion, 8, 8)
);

#[derive(MemSize, MemDbg)]
union TestUnionDeep<'a> {
    b: &'a TestUnion,
}

#[derive(MemSize, MemDbg)]
union TestUnionDeepMut<'a> {
    b: &'a mut TestUnion,
}

#[test]
fn test_single_field_union_follow_ref() {
    let mut test_union = TestUnion::default();
    let test_union_deep = TestUnionDeep { b: &test_union };

    // We check that the shallow size of the test union deep is the
    // size of a reference (i.e. an usize).
    assert_eq!(
        <TestUnionDeep as MemSize>::mem_size(&test_union_deep, SizeFlags::default()),
        core::mem::size_of::<usize>(),
    );

    // We check that the deep size of the test union deep is the
    // size of a reference plus the size of the test union.
    assert_eq!(
        <TestUnionDeep as MemSize>::mem_size(&test_union_deep, SizeFlags::FOLLOW_REFS),
        core::mem::size_of::<usize>()
            + <TestUnion as MemSize>::mem_size(&test_union, SizeFlags::default()),
    );

    let test_union_deep_mut = TestUnionDeepMut { b: &mut test_union };

    // We check that the shallow size of the test union mut is the
    // size of a reference (i.e. an usize)
    assert_eq!(
        <TestUnionDeepMut as MemSize>::mem_size(&test_union_deep_mut, SizeFlags::default()),
        core::mem::size_of::<usize>(),
    );

    // We check that the deep size of the test union deep mut is the
    // size of a reference plus the size of the test union.
    assert_eq!(
        <TestUnionDeepMut as MemSize>::mem_size(&test_union_deep_mut, SizeFlags::FOLLOW_REFS),
        core::mem::size_of::<usize>()
            + <TestUnion as MemSize>::mem_size(&test_union, SizeFlags::default()),
    );
}
