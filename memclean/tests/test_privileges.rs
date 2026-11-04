use memclean::core::privileges;

#[test]
fn test_is_elevated_callable_without_panic() {
    let elevated = privileges::is_elevated();
    // On Windows, is_elevated returns a valid boolean without panicking or leaking handles
    println!("Process elevation status: {}", elevated);
}
