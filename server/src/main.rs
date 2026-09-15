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

use actix::{Actor, Addr};
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use actors::ClientActor;
use actors::RoomManagerActor;

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
                    .default_handler(
                        web::to(|| async {
                            actix_files::NamedFile::open_async("./dist/index.html").await
                        }),
                    ),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
