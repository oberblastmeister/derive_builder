#[test]
fn trybuild() {
    let t = trybuild::TestCases::new();
    t.pass("trybuild/01-parse.rs");
    t.compile_fail("trybuild/test_enum_fail.rs");
    t.compile_fail("trybuild/test_nonnamed_fail.rs");
    t.pass("trybuild/02-create-builder.rs");
    t.pass("trybuild/03-call-setters.rs");
    t.pass("trybuild/04-call-build.rs");
    t.pass("trybuild/05-method-chaining.rs");
    t.pass("trybuild/06-optional-field.rs");
    t.pass("trybuild/07-repeated-field.rs");
    t.compile_fail("trybuild/08-unrecognized-attribute.rs");
    t.pass("trybuild/09-redefined-prelude-types.rs");
}
