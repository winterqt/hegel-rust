use super::{labels, BasicGenerator, Generate, TestCaseData};
use crate::cbor_utils::cbor_map;
use ciborium::Value;
use std::marker::PhantomData;

pub struct ArrayGenerator<G, T, const N: usize> {
    element: G,
    _phantom: PhantomData<fn() -> T>,
}

impl<G, T, const N: usize> ArrayGenerator<G, T, N> {
    pub fn new(element: G) -> Self {
        ArrayGenerator {
            element,
            _phantom: PhantomData,
        }
    }
}

pub fn arrays<G: Generate<T> + Send + Sync, T, const N: usize>(
    element: G,
) -> ArrayGenerator<G, T, N> {
    ArrayGenerator::new(element)
}

impl<G: Generate<T> + Send + Sync, T, const N: usize> Generate<[T; N]> for ArrayGenerator<G, T, N> {
    fn do_draw(&self, data: &TestCaseData) -> [T; N] {
        if let Some(basic) = self.as_basic() {
            basic.do_draw(data)
        } else {
            data.span_group(labels::TUPLE, || {
                std::array::from_fn(|_| self.element.do_draw(data))
            })
        }
    }

    fn as_basic(&self) -> Option<BasicGenerator<'_, [T; N]>> {
        let basic = self.element.as_basic()?;

        let elements = Value::Array((0..N).map(|_| basic.schema().clone()).collect());
        let schema = cbor_map! {
            "type" => "tuple",
            "elements" => elements
        };

        Some(BasicGenerator::new(schema, move |raw| {
            let arr = match raw {
                Value::Array(arr) => arr,
                _ => panic!("Expected array from tuple schema, got {:?}", raw),
            };
            assert_eq!(arr.len(), N);
            let mut iter = arr.into_iter();
            std::array::from_fn(|_| basic.parse_raw(iter.next().unwrap()))
        }))
    }
}
