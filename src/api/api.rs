use std::net::Ipv4Addr;

use rocket::{get, serde::json::Json, Build, Config, Rocket, Route};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ServiceHealth {
    health: String,
}

#[get("/healthcheck")]
pub fn healthcheck() -> Json<ServiceHealth> {
    Json(ServiceHealth {
        health: "OK".to_string(),
    })
}

pub async fn lezgong(routes: Vec<Route>, port: u16) -> Rocket<Build> {
    rocket::build()
        .configure(Config {
            port,
            address: "0.0.0.0"
                .parse::<Ipv4Addr>()
                .unwrap_or(Ipv4Addr::new(0, 0, 0, 0))
                .into(),
            log_level: rocket::config::LogLevel::Normal,
            ..Config::default()
        })
        .mount("/patishie", routes)
}
