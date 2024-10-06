#[macro_export]
macro_rules! log_and_return_error {
    ($err:expr) => {{
        log::error!("{}", $err.to_string());
        return Err($err);
    }};
}
