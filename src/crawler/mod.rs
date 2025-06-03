pub mod leetcode;

pub async fn retry<F, Fut, T, E>(times: usize, mut f: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    for attempt in 1..=times {
        match f().await {
            Ok(val) => return Ok(val),
            Err(err) if attempt == times => return Err(err),
            Err(_) => {}
        }
    }

    unreachable!("retry loop should always return or err out before this")
}
