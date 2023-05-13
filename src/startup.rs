use std::{io, net::TcpListener};

use actix_web::{dev::Server, web::Data, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::{configuration::Settings, email_client::EmailClient, routes};

pub struct Application {
    pub server: Server,
    pub port: u16,
}

impl Application {
    pub async fn build(configuration: Settings) -> eyre::Result<Application> {
        let connection_pool = PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(configuration.database.withdb());
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.application.base_url,
        )?;
        Ok(Self { server, port })
    }

    pub async fn run_util_stopped(self) -> Result<(), io::Error> {
        self.server.await
    }

    pub async fn port(&self) -> u16 {
        self.port
    }
}

pub fn run(
    listener: TcpListener,
    pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> eyre::Result<Server> {
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(routes::health_check)
            .service(routes::subscribe)
            .service(routes::subscribe_confirm)
            .service(routes::publish_newsletter)
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(email_client.clone()))
            .app_data(Data::new(base_url.clone()))
    })
    .listen(listener)?
    .run())
}
