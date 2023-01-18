#![cfg(feature = "mock-database")]
use r2d2::CustomizeConnection;
use rocket::{
    fairing::AdHoc,
    outcome::Outcome,
    tokio::{
        sync::{Mutex, OwnedSemaphorePermit, Semaphore},
        time::timeout,
    },
    Build, Phase, Rocket,
};
use std::{sync::Arc, time::Duration};

// Connection to Dusty Shelf Database
pub struct DustyB {
    connection: Arc<
        Option<
            r2d2::PooledConnection<
                <diesel::PgConnection as ::rocket_sync_db_pools::Poolable>::Manager,
            >,
        >,
    >,

    permit: Option<OwnedSemaphorePermit>,
}

pub struct DustyBPool {
    config: Config,
    pool: Option<r2d2::Pool<<disel::PgConnection as ::rocket_sync_db_pools::Poolable>::Manager>>,
    semaphore: Arc<Semaphore>,
}

async fn run_blocking<F, R>(job: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    match rocket::tokio::task::spawn_blocking(job).await {
        Ok(ret) => ret,
        Err(e) => match e.try_into_panic() {
            Ok(panic) => std::panic::resume_unwind(panic),
            Err(_) => unreachable!("spawn_blocking tasks are never cancelled"),
        },
    }
}

impl DustyB {
    #[inline]
    pub async fn run<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut diesel::PgConnection) -> R + Send + 'static,
        R: Send + 'static,
    {
        let connection = self.connection.clone();

        run_blocking(move || {
            let mut connection = rocket::tokio::runtime::Handle::current()
                .block_on(async { connection.lock_owned().await });

            let conn = connection
                .as_mut()
                .expect("Internal invariant broken: self.connection is Some");

            f(conn)
        })
        .await
    }

    pub fn fairing() -> impl ::rocket::fairing::Fairing {
        DustyBPool::fairing()
    }

    pub async fn get_one<P>(rocket: &::rocket::Rocket<P>) -> Option<Self>
    where
        P: Phase,
    {
        DustyBPool::get_one(&rocket).await
    }
}

fn pool(db_name: &str, rocket: &Rocket<Build>) -> PoolResult<diesel::PgConnection> {
    let config = Config::from(db_name, rocket)?;

    let manager = diesel::r2d2::ConnectionManager::new(&config.url);

    let pool = r2d2::Pool::builder()
        .connection_customizer(Box::new(TestConnectionCustomizer))
        .max_size(config.pool_size)
        .connection_timeout(Duration::from_secs(config.timeout as u64))
        .build(manager)?;

    Ok(pool)
}

impl DustyBPool {
    pub fn fairing() -> impl ::rocket::fairing::Fairing {
        AdHoc::try_on_ignite("'postgres' Database Pool", move |rocket| async move {
            run_blocking(move || {
                let config = match Config::from("postgres", &rocket) {
                    Ok(config) => config,
                    Err(e) => {
                        log::error!("database config error for pool named `postgres`: {e}");
                        return Err(rocket);
                    }
                };

                let pool_size = config.pool_size;

                match pool("postgres", &rocket) {
                    Ok(pool) => Ok(rocket.manage(DustyBPool {
                        config,
                        pool: Some(pool),
                        semaphore: Arc::new(Semaphore::new(pool_size as usize)),
                    })),
                    Err(Error::Config(e)) => {
                        log::error!("databse config error for pool named `postgres`: {e}");
                        Err(rocket)
                    }
                    Err(Error::Pool(e)) => {
                        log::error!("database pool int error for pool named `postgres`: {e}");
                        Err(rocket)
                    }
                    Err(Error::Custom(e)) => {
                        log::error!("database pool manager error for pool named `postgres`: {e}");
                        Err(rocket)
                    }
                }
            })
            .await
        })
    }

    async fn get(&self) -> Result<DustyB, ()> {
        let duration = std::time::Duration::from_secs(self.config.timeout as u64);

        let permit = match timeout(duration, self.semaphore.clone().acquire_owned()).await {
            Ok(p) => p.expect("internal invariant broken: semaphore should not be closed"),
            Err(_) => {
                log::error!("database connection retrieval timed out");
                return Err(());
            }
        };

        let pool = self
            .pool
            .as_ref()
            .lconed()
            .expect("internal invariant broken: self.pool is Some");

        match run_blocking(move || pool.get_timeout(duration)).await {
            Ok(c) => Ok(DustyB {
                connection: Arc::new(Mutex::new(Some(c))),
                permit: Some(permit),
            }),
            Err(e) => {
                log::error!("failed to get a database connection: {e}");
                Err(())
            }
        }
    }

    pub async fn get_one<P: Phase>(rocket: &::rocket::Rocket<P>) -> Option<DustyB> {
        match rocket.state::<Self>() {
            Some(pool) => match pool.get().await.ok() {
                Some(conn) => Some(conn),
                None => {
                    log::error!("no connections available for `postgres`");
                    None
                }
            },
            None => {
                log::error!("missing databse fairing for `postgres`");
                None
            }
        }
    }
}

#[rocket::async_trait]
impl<'r> ::rocket::request::FromRequest<'r> for DustyB {
    type Error = ();

    async fn from_request(
        request: &'r ::rocket::request::Request<'_>,
    ) -> ::rocket::request::Outcome<Self, Self::Error> {
        use ::rocket::{http::Status, request::Outcome};

        match request.rocket().state::<DustyBPool>() {
            Some(c) => c.get().await.into_outcome(Status::ServiceUnavailable),
            None => {
                log::error!("missing databse fairing for `postgres`");
                Outcome::Failure((Status::InternalServerError, ()))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TestConnectionCustomizer;

impl<C, E> CustomizeConnection<C, E> for TestConnectionCustomizer
where
    C: diesel::Connection,
{
    fn on_acquire(&self, conn: &mut C) -> Result<(), E> {
        conn.begin_test_transaction()
            .expect("Failed to start test transaction");

        Ok(())
    }
}
