use seu::DataType;

#[test]
fn duplicate_names() {
    #[expect(dead_code, reason = "Compilation-only test")]
    #[derive(DataType)]
    enum TestDuplicate {
        First { bar: usize },
        Second { bar: u64 },
    }
}
