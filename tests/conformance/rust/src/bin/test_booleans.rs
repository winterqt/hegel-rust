use hegel::gen::{self, Generate};
use hegel::{hegel_with_options, HegelOptions};
use hegel_conformance::{get_test_cases, write};
use serde::Serialize;

#[derive(Serialize)]
struct Metrics {
    value: bool,
}

fn main() {
    // booleans takes no params, so we ignore argv[1]

    hegel_with_options(
        || {
            let value = gen::booleans().generate();
            write(&Metrics { value });
        },
        HegelOptions::new().with_test_cases(get_test_cases()),
    );
}
