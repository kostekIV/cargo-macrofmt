// line length over 80 - should format
#[instrument(level = "debug", target = LOG_TARGET, name = "process-request", skip_all)]
async fn process_request(&mut self, msg: Request<T>) -> Result<(), ProcessError> {
    Ok(())
}

// line length less than 80 - should not format
#[instrument(level = "debug", target = LOG_TARGET, name = "process-data")]
async fn process_data(&mut self, msg: Request<T>) -> Result<(), ProcessError> {
    Ok(())
}
