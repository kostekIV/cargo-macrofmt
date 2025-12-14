// line length over 80 - should format
#[instrument(
    level = "debug",
    target = LOG_TARGET,
    name = "process-request",
    skip_all,
)]
async fn process_request(&mut self, msg: Request<T>) -> Result<(), ProcessError> {
    Ok(())
}

// line length less than 80 - should not format
#[instrument(level = "debug", target = LOG_TARGET)]
async fn process_data(&mut self, msg: Request<T>) -> Result<(), ProcessError> {
    Ok(())
}

// other macros - should never format regardless of length
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
struct LongStruct {
    field: String,
}

#[cfg(all(test, feature = "integration", target_os = "linux", target_arch = "x86_64"))]
mod tests {
    fn test() {}
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn async_test() {}

#[test]
#[should_panic(expected = "some very long panic message that exceeds the line length limit")]
fn panic_test() {}
