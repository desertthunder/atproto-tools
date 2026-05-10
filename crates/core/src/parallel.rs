use std::{error::Error, fmt, future::Future, time::Duration};

use tokio::{task::JoinSet, time::sleep};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParallelConfig {
    pub max_parallel: usize,
    pub start_delay: Duration,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self { max_parallel: 8, start_delay: Duration::from_millis(50) }
    }
}

pub async fn run_parallel_rate_limited<I, O, E, F, Fut>(
    items: Vec<I>, config: ParallelConfig, worker: F,
) -> Result<Vec<O>, ParallelTaskError<E>>
where
    I: Send + 'static,
    O: Send + 'static,
    E: Send + 'static,
    F: Fn(I) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<O, E>> + Send + 'static,
{
    if config.max_parallel == 0 {
        return Err(ParallelTaskError::InvalidMaxParallel);
    }

    let len = items.len();
    let mut results = (0..len).map(|_| None).collect::<Vec<_>>();
    let mut tasks = JoinSet::new();

    for (index, item) in items.into_iter().enumerate() {
        while tasks.len() >= config.max_parallel {
            collect_next(&mut tasks, &mut results).await?;
        }

        if index > 0 && !config.start_delay.is_zero() {
            sleep(config.start_delay).await;
        }

        let worker = worker.clone();
        tasks.spawn(async move { (index, worker(item).await) });
    }

    while !tasks.is_empty() {
        collect_next(&mut tasks, &mut results).await?;
    }

    Ok(results
        .into_iter()
        .map(|result| result.expect("all parallel task results were collected"))
        .collect())
}

async fn collect_next<O, E>(
    tasks: &mut JoinSet<(usize, Result<O, E>)>, results: &mut [Option<O>],
) -> Result<(), ParallelTaskError<E>>
where
    O: Send + 'static,
    E: Send + 'static,
{
    let Some(joined) = tasks.join_next().await else {
        return Ok(());
    };

    let (index, result) = joined.map_err(ParallelTaskError::Join)?;
    results[index] = Some(result.map_err(ParallelTaskError::Task)?);
    Ok(())
}

#[derive(Debug)]
pub enum ParallelTaskError<E> {
    InvalidMaxParallel,
    Join(tokio::task::JoinError),
    Task(E),
}

impl<E> fmt::Display for ParallelTaskError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMaxParallel => formatter.write_str("max_parallel must be greater than zero"),
            Self::Join(source) => write!(formatter, "parallel task failed to join: {source}"),
            Self::Task(source) => write!(formatter, "parallel task failed: {source}"),
        }
    }
}

impl<E> Error for ParallelTaskError<E>
where
    E: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidMaxParallel => None,
            Self::Join(source) => Some(source),
            Self::Task(source) => Some(source),
        }
    }
}
