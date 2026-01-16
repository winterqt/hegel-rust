use hegel::gen::{self, Generate};

#[test]
fn test_integers_i8_within_bounds() {
    hegel::hegel(|| {
        let x = gen::integers::<i8>().generate();
        assert!(x >= i8::MIN && x <= i8::MAX);
    })
}

#[test]
fn test_integers_i16_within_bounds() {
    hegel::hegel(|| {
        let x = gen::integers::<i16>().generate();
        assert!(x >= i16::MIN && x <= i16::MAX);
    })
}

#[test]
fn test_integers_i32_within_bounds() {
    hegel::hegel(|| {
        let x = gen::integers::<i32>().generate();
        assert!(x >= i32::MIN && x <= i32::MAX);
    })
}

#[test]
fn test_integers_i64_within_bounds() {
    hegel::hegel(|| {
        let x = gen::integers::<i64>().generate();
        assert!(x >= i64::MIN && x <= i64::MAX);
    })
}

#[test]
fn test_integers_u8_within_bounds() {
    hegel::hegel(|| {
        let x = gen::integers::<u8>().generate();
        assert!(x >= u8::MIN && x <= u8::MAX);
    })
}

#[test]
fn test_integers_u16_within_bounds() {
    hegel::hegel(|| {
        let x = gen::integers::<u16>().generate();
        assert!(x >= u16::MIN && x <= u16::MAX);
    })
}

#[test]
fn test_integers_u32_within_bounds() {
    hegel::hegel(|| {
        let x = gen::integers::<u32>().generate();
        assert!(x >= u32::MIN && x <= u32::MAX);
    })
}

#[test]
fn test_integers_u64_within_bounds() {
    hegel::hegel(|| {
        let x = gen::integers::<u64>().generate();
        assert!(x >= u64::MIN && x <= u64::MAX);
    })
}
