use hegel::generators::{self, Generator};
use hegel::TestCase;

#[hegel::test]
fn boxed_generator_with_weird_lifetimes(tc: TestCase) {
    // This tests that we can created boxed generators boxed
    // generators whose lifetimes may not outlive the test.
    let x = vec!["foo", "bar", "baz"];

    let ix = generators::integers().min_value(0).max_value(2);

    // Generator for a reference into x, which necessarily
    // means that `refs` may not outlive `x`.
    let refs = ix.map(|i| &x[i]).boxed();

    let t = tc.draw(refs);

    assert!(t.len() == 3);
}

#[hegel::test]
fn default_can_infer_through_draw(tc: TestCase) {
    let _: i32 = tc.draw(generators::default());
}
