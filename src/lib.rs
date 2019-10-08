#[macro_use]
extern crate log;

use futures::{future::Future, stream::Stream};
use signal_hook::{self, iterator::Signals};
use std::io::Error;
use std::marker::PhantomData;
use tokio;
use tokio_threadpool::ThreadPool;

pub trait ServiceControl<T> {
    fn create_and_start(service_config: &'static T, thread_pool: &'static ThreadPool) -> Self;
    fn shutdown(self);
}

pub struct ServiceHost<T, S> {
    service_container: S,
    phantom: PhantomData<T>,
}

impl<T, S> ServiceHost<T, S>
where
    T: Send + 'static,
    S: ServiceControl<T> + Send + 'static,
{
    pub fn create_and_start(service_config: &'static T, thread_pool: &'static ThreadPool) -> Self {
        Self {
            service_container: S::create_and_start(service_config, thread_pool),
            phantom: PhantomData,
        }
    }
}

impl<T, S> ServiceHost<T, S>
where
    T: Send + 'static,
    S: ServiceControl<T> + Send + 'static,
{
    pub fn wait_for_signal(self) -> Result<(), Error> {
        let wait_signal = Signals::new(&[signal_hook::SIGINT, signal_hook::SIGTERM])?
            .into_async()?
            .into_future()
            .map(move |_| self.service_container.shutdown())
            .map_err(|e| {
                let error_message = format!("Signal panic!\n{}", e.0);
                error!("{}", &error_message);
                panic!("{}", &error_message);
            });
        info!("Waiting for exit signal...");
        tokio::run(wait_signal);
        Ok(())
    }
}
