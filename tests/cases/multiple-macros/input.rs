// Both instrument and test_macro should format when over 80 chars
#[instrument(level = "debug", target = LOG_TARGET, name = "process-request", skip_all)]
async fn process_request(&mut self, msg: Request<T>) -> Result<(), ProcessError> {
    Ok(())
}

#[test_macro(param1 = LOG_TARGET, param2 = "value2", param3 = "value3", param4 = "value4")]
fn test_function() {
    assert!(true);
}

// Under 80 chars - should not format
#[instrument(level = "debug", skip_all)]
async fn short_fn() -> Result<()> {
    Ok(())
}

#[test_macro(param1 = "short")]
fn short_test() {
    assert!(true);
}
