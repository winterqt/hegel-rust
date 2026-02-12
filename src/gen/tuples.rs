use super::{group, labels, BasicGenerator, Generate, RawParse};
use crate::cbor_helpers::{cbor_array, cbor_map};
use ciborium::Value;
use std::mem::MaybeUninit;

pub struct Tuple2Generator<G1, G2> {
    gen1: G1,
    gen2: G2,
}

impl<T1, T2, G1, G2> Generate<(T1, T2)> for Tuple2Generator<G1, G2>
where
    G1: Generate<T1>,
    G2: Generate<T2>,
{
    fn generate(&self) -> (T1, T2) {
        if let Some(basic) = self.as_basic() {
            basic.generate()
        } else {
            group(labels::TUPLE, || {
                let v1 = self.gen1.generate();
                let v2 = self.gen2.generate();
                (v1, v2)
            })
        }
    }

    fn as_basic(&self) -> Option<BasicGenerator<'_, (T1, T2)>> {
        let basic1 = self.gen1.as_basic()?;
        let basic2 = self.gen2.as_basic()?;

        let schema = cbor_map! {
            "type" => "tuple",
            "elements" => cbor_array![basic1.schema().clone(), basic2.schema().clone()]
        };

        let raw1 = basic1.into_raw();
        let raw2 = basic2.into_raw();

        let writer: Box<dyn Fn(Value, *mut u8) + Send + Sync + '_> =
            Box::new(move |raw, out_ptr| {
                let arr = match raw {
                    Value::Array(arr) => arr,
                    _ => panic!("Expected array from tuple schema, got {:?}", raw),
                };
                let mut iter = arr.into_iter();

                let mut v1_out = MaybeUninit::<T1>::uninit();
                unsafe {
                    raw1.invoke(
                        iter.next().expect("tuple missing element 0"),
                        v1_out.as_mut_ptr() as *mut u8,
                    )
                };
                let v1 = unsafe { v1_out.assume_init() };

                let mut v2_out = MaybeUninit::<T2>::uninit();
                unsafe {
                    raw2.invoke(
                        iter.next().expect("tuple missing element 1"),
                        v2_out.as_mut_ptr() as *mut u8,
                    )
                };
                let v2 = unsafe { v2_out.assume_init() };

                unsafe { std::ptr::write(out_ptr as *mut (T1, T2), (v1, v2)) };
            });

        Some(unsafe {
            BasicGenerator::from_raw(RawParse {
                schema,
                call: writer,
            })
        })
    }
}

pub fn tuples<T1, T2, G1: Generate<T1>, G2: Generate<T2>>(
    gen1: G1,
    gen2: G2,
) -> Tuple2Generator<G1, G2> {
    Tuple2Generator { gen1, gen2 }
}

pub struct Tuple3Generator<G1, G2, G3> {
    gen1: G1,
    gen2: G2,
    gen3: G3,
}

impl<T1, T2, T3, G1, G2, G3> Generate<(T1, T2, T3)> for Tuple3Generator<G1, G2, G3>
where
    G1: Generate<T1>,
    G2: Generate<T2>,
    G3: Generate<T3>,
{
    fn generate(&self) -> (T1, T2, T3) {
        if let Some(basic) = self.as_basic() {
            basic.generate()
        } else {
            group(labels::TUPLE, || {
                let v1 = self.gen1.generate();
                let v2 = self.gen2.generate();
                let v3 = self.gen3.generate();
                (v1, v2, v3)
            })
        }
    }

    fn as_basic(&self) -> Option<BasicGenerator<'_, (T1, T2, T3)>> {
        let basic1 = self.gen1.as_basic()?;
        let basic2 = self.gen2.as_basic()?;
        let basic3 = self.gen3.as_basic()?;

        let schema = cbor_map! {
            "type" => "tuple",
            "elements" => cbor_array![
                basic1.schema().clone(),
                basic2.schema().clone(),
                basic3.schema().clone()
            ]
        };

        let raw1 = basic1.into_raw();
        let raw2 = basic2.into_raw();
        let raw3 = basic3.into_raw();

        let writer: Box<dyn Fn(Value, *mut u8) + Send + Sync + '_> =
            Box::new(move |raw, out_ptr| {
                let arr = match raw {
                    Value::Array(arr) => arr,
                    _ => panic!("Expected array from tuple schema, got {:?}", raw),
                };
                let mut iter = arr.into_iter();

                let mut v1_out = MaybeUninit::<T1>::uninit();
                unsafe {
                    raw1.invoke(
                        iter.next().expect("tuple missing element 0"),
                        v1_out.as_mut_ptr() as *mut u8,
                    )
                };
                let v1 = unsafe { v1_out.assume_init() };

                let mut v2_out = MaybeUninit::<T2>::uninit();
                unsafe {
                    raw2.invoke(
                        iter.next().expect("tuple missing element 1"),
                        v2_out.as_mut_ptr() as *mut u8,
                    )
                };
                let v2 = unsafe { v2_out.assume_init() };

                let mut v3_out = MaybeUninit::<T3>::uninit();
                unsafe {
                    raw3.invoke(
                        iter.next().expect("tuple missing element 2"),
                        v3_out.as_mut_ptr() as *mut u8,
                    )
                };
                let v3 = unsafe { v3_out.assume_init() };

                unsafe { std::ptr::write(out_ptr as *mut (T1, T2, T3), (v1, v2, v3)) };
            });

        Some(unsafe {
            BasicGenerator::from_raw(RawParse {
                schema,
                call: writer,
            })
        })
    }
}

pub fn tuples3<T1, T2, T3, G1: Generate<T1>, G2: Generate<T2>, G3: Generate<T3>>(
    gen1: G1,
    gen2: G2,
    gen3: G3,
) -> Tuple3Generator<G1, G2, G3> {
    Tuple3Generator { gen1, gen2, gen3 }
}
