//! Every file under `tests/pass` must compile and run, every file under
//! `tests/compile_fail` must fail with exactly its `.stderr`.

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/*.rs");
    t.compile_fail("tests/compile_fail/*.rs");
}
