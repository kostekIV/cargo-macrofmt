// Macro as last segment in various paths
#[tracing::instrument(
    level = "debug",
    target = LOG_TARGET,
    name = "handle-event",
    skip_all,
)]
async fn handle_event(&mut self) -> Result<()> {
    Ok(())
}

#[custom::testing::test_macro(
    param1 = "value1",
    param2 = "value2",
    param3 = "value3",
)]
fn custom_test() {
    assert!(true);
}

#[some::deeply::nested::path::instrument(
    level = "info",
    name = "nested",
    skip_all,
)]
fn nested_instrument() {
    println!("nested");
}

// Should not format - different macro name
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn async_test() {}

// Short versions - should not format
#[tracing::instrument(level = "debug")]
async fn short_tracing() -> Result<()> {
    Ok(())
}

#[test_macro(short = "param")]
fn short_custom() {}
