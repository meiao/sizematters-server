/*
 * SizeMatters - a ticket sizing util
 * Copyright (C) 2025 Andre Onuki
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

mod actors;
pub mod avatar;

use actix::{Actor, Addr};
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actors::ClientActor;
use actors::RoomManagerActor;
use opentelemetry::global as otel;
use opentelemetry_otlp::tonic_types::transport::ClientTlsConfig;
use opentelemetry_otlp::{Protocol, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::metrics::{SdkMeterProvider, Temporality};
use opentelemetry_sdk::Resource;

/// do websocket handshake and start `MyWebSocket` actor
async fn ws_index(
    r: HttpRequest,
    stream: web::Payload,
    room_manager: web::Data<Addr<RoomManagerActor>>,
) -> Result<HttpResponse, Error> {
    let room_manager_addr = room_manager.get_ref().clone();
    ws::start(ClientActor::new(room_manager_addr), &r, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let room_manager = RoomManagerActor::new().start();

    initialize_o11y();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(room_manager.clone()))
            .wrap(middleware::Logger::default())
            // websocket route
            .service(web::resource("/ws").route(web::get().to(ws_index)))
            // serve frontend
            .service(
                actix_files::Files::new("/", "./dist")
                    .index_file("index.html")
                    .default_handler(web::to(|| async {
                        actix_files::NamedFile::open("./dist/index.html")
                    })),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

fn initialize_o11y() {
    if std::env::var_os("OTEL_EXPORTER_OTLP_HEADERS").is_none() {
        eprintln!("OTEL_EXPORTER_OTLP_HEADERS not set, telemetry disabled");
        return;
    }

    let service_name = std::env::var("SERVICE_NAME").unwrap_or("sizematters".to_string());

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint("https://otlp.nr-data.net")
        .with_tls_config(ClientTlsConfig::new().with_enabled_roots())
        .with_temporality(Temporality::Delta)
        .with_protocol(Protocol::Grpc)
        .build()
        .expect("Failed to build exporter");
    let provider = SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(Resource::builder().with_service_name(service_name).build())
        .build();
    otel::set_meter_provider(provider);
}
