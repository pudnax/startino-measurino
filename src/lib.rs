#[macro_export]
macro_rules! measure {
    ( $reason:expr, $tt:block ) => {{
        let it = std::time::Instant::now();
        let ret = $tt;
        let elapsed = it.elapsed().as_secs_f64() * 1000.;
        println!("[{elapsed:12.3} ms] {}", $reason);
        ret
    }};
}
