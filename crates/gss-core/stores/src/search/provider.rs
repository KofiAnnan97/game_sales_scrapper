use errors::api::ApiError;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{AcquireError, Semaphore};

#[derive(Debug, Clone)]
pub struct SearchProvider {
    pub client: reqwest::Client,
    pub semaphore: Arc<Semaphore>,
}

pub enum SearchOutput<T> {
    Result(Result<T, ApiError>),
    Option(Option<T>),
}

pub enum SearchError<E> {
    SemaphoreError(AcquireError),
    StoreError(E),
}

impl SearchProvider {
    pub fn new(max_concurrency: usize) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .user_agent(format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(5))
            .read_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(10)
            .build()?;

        Ok(Self {
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
            client,
        })
    }

    pub async fn option_search<F, Fut, T>(&self, search: F) -> Option<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Option<T>>,
    {
        let _permit = self.semaphore.acquire().await.ok()?;
        search().await
    }

    pub async fn result_search<F, Fut, T, E>(&self, search: F) -> Result<T, SearchError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(SearchError::SemaphoreError)?;
        search().await.map_err(SearchError::StoreError)
    }
}
