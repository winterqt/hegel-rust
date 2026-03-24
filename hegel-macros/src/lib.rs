mod composite;
mod enum_gen;
mod hegel_test;
mod stateful;
mod struct_gen;
mod utils;

use proc_macro::TokenStream;
use syn::{Data, DeriveInput, ItemFn, ItemImpl, parse_macro_input};

/// Derive a generator for a struct or enum.
///
/// This implements [`DefaultGenerator`](hegel::generators::DefaultGenerator) for the type,
/// allowing it to be used with [`default`](hegel::generators::default) via `default::<T>()`.
///
/// For structs, the generated generator has:
/// - `<field>(generator)` - builder method to customize each field's generator
///
/// For enums, the generated generator has:
/// - `default_<VariantName>()` - methods returning default variant generators
/// - `<VariantName>(generator)` - builder methods to customize variant generation
///
/// # Struct Example
///
/// ```ignore
/// use hegel::DefaultGenerator;
/// use hegel::generators::{self, DefaultGenerator as _, Generator as _};
///
/// #[derive(DefaultGenerator)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// #[hegel::test]
/// fn generates_people(tc: hegel::TestCase) {
///     let generator = generators::default::<Person>()
///         .age(generators::integers::<u32>().min_value(0).max_value(120));
///     let person: Person = tc.draw(generator);
/// }
/// ```
///
/// # Enum Example
///
/// ```ignore
/// use hegel::DefaultGenerator;
/// use hegel::generators::{self, DefaultGenerator as _, Generator as _};
///
/// #[derive(DefaultGenerator)]
/// enum Status {
///     Pending,
///     Active { since: String },
///     Error { code: i32, message: String },
/// }
///
/// #[hegel::test]
/// fn generates_statuses(tc: hegel::TestCase) {
///     let generator = generators::default::<Status>()
///         .Active(
///             generators::default::<Status>()
///                 .default_Active()
///                 .since(generators::text().max_size(20))
///         );
///     let status: Status = tc.draw(generator);
/// }
/// ```
#[proc_macro_derive(DefaultGenerator)]
pub fn derive_generator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match &input.data {
        Data::Struct(data) => struct_gen::derive_struct_generator(&input, data),
        Data::Enum(data) => enum_gen::derive_enum_generator(&input, data),
        Data::Union(_) => syn::Error::new_spanned(&input, "Generator cannot be derived for unions")
            .to_compile_error()
            .into(),
    }
}

/// Mark a test function as a Hegel property-based test.
///
/// Wraps the function body in `Hegel::new(|tc: TestCase| { ... }).run()`. The function
/// must take exactly one parameter of type `hegel::TestCase`, and use `tc.draw()` to
/// generate values. The `#[test]` attribute is added automatically and must not be
/// present on the function.
///
/// Optionally accepts settings as `key = value` pairs corresponding to
/// methods on [`Settings`](hegel::Settings):
///
/// ```ignore
/// #[hegel::test]
/// fn my_test(tc: hegel::TestCase) {
///     let x: i32 = tc.draw(generators::integers());
///     assert!(x + 0 == x);
/// }
///
/// #[hegel::test(test_cases = 500)]
/// fn my_configured_test(tc: hegel::TestCase) {
///     let x: i32 = tc.draw(generators::integers());
///     assert!(x + 0 == x);
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    hegel_test::expand_test(attr.into(), item.into()).into()
}

/// Define a composite generator from a function.
///
/// The first parameter must be `tc: TestCase` and is passed automatically
/// when the generator is drawn. Any additional parameters become parameters
/// of the returned factory function. The function must have an explicit
/// return type.
///
/// ```ignore
/// use hegel::generators;
///
/// #[hegel::composite]
/// fn sorted_vec(tc: hegel::TestCase, min_len: usize) -> Vec<i32> {
///     let mut v: Vec<i32> = tc.draw(generators::vecs(generators::integers()).min_size(min_len));
///     v.sort();
///     v
/// }
///
/// #[hegel::test]
/// fn test_sorted(tc: hegel::TestCase) {
///     let v = tc.draw(sorted_vec(3));
///     assert!(v.len() >= 3);
///     assert!(v.windows(2).all(|w| w[0] <= w[1]));
/// }
/// ```
#[proc_macro_attribute]
pub fn composite(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    composite::expand_composite(input).into()
}

/// Derive a [`StateMachine`](hegel::stateful::StateMachine) implementation from an `impl` block.
///
/// Methods annotated with `#[rule]` become rules (actions applied to the state machine)
/// and methods annotated with `#[invariant]` become invariants (checked after each
/// successful rule). Rules take `&mut self` and a `TestCase`; invariants take `&self`
/// and a `TestCase`.
///
/// ```ignore
/// use hegel::TestCase;
/// use hegel::generators::integers;
///
/// struct IntegerStack {
///     stack: Vec<i32>,
/// }
///
/// #[hegel::state_machine]
/// impl IntegerStack {
///     #[rule]
///     fn push(&mut self, tc: TestCase) {
///         let element = tc.draw(integers::<i32>());
///         self.stack.push(element);
///     }
///
///     #[rule]
///     fn pop(&mut self, _: TestCase) {
///         self.stack.pop();
///     }
///
///     #[rule]
///     fn pop_push(&mut self, tc: TestCase) {
///         let element = tc.draw(integers::<i32>());
///         let initial = self.stack.clone();
///         self.stack.push(element);
///         let popped = self.stack.pop().unwrap();
///         assert_eq!(popped, element);
///         assert_eq!(self.stack, initial);
///     }
///
///     #[rule]
///     fn push_pop(&mut self, tc: TestCase) {
///         let initial = self.stack.clone();
///         let element = self.stack.pop();
///         tc.assume(element.is_some());
///         let element = element.unwrap();
///         self.stack.push(element);
///         assert_eq!(self.stack, initial);
///     }
/// }
///
/// #[hegel::test]
/// fn test_integer_stack(tc: TestCase) {
///     let stack = IntegerStack { stack: Vec::new() };
///     hegel::stateful::run(stack, tc);
/// }
/// ```
#[proc_macro_attribute]
pub fn state_machine(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let block = parse_macro_input!(item as ItemImpl);
    stateful::expand_state_machine(block).into()
}
