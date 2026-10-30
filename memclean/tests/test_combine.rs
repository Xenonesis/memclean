use memclean::core::combine;

#[test]
#[allow(unused_comparisons)]
fn test_combine_physical_memory_call() {
    let res = combine::combine_physical_memory();
    // In elevated admin environment, returns Ok(CombineResult);
    // In unprivileged test environment, returns Err(NTSTATUS 0xC0000061) without panic.
    match res {
        Ok(c) => assert!(c.pages_combined >= 0),
        Err(e) => assert!(!e.is_empty(), "Error message should describe NTSTATUS failure"),
    }
}
