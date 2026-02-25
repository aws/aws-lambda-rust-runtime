// Logs using tracing `error!` if a dispatcher is set, otherwise falls back to `eprintln!`.
macro_rules! log_or_print {
    (tracing: $tracing_expr:expr, fallback: $fallback_expr:expr) => {
        if tracing::dispatcher::has_been_set() {
            $tracing_expr;
        } else {
            $fallback_expr;
        }
    };
}
