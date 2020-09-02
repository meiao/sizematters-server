/*
 * SizeMatters - a ticket sizing util
 * Copyright (C) 2020 Andre Onuki
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
mod data;

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
    //println!("{:?}", r);
    let room_manager_addr = room_manager.get_ref().clone();
    let res = ws::start(ClientActor::new(room_manager_addr), &r, stream);
    //println!("{:?}", res);
    res
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    let room_manager = RoomManagerActor::new().start();

    HttpServer::new(move || {
        App::new()
            .data(room_manager.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            // websocket route
            .service(web::resource("/").route(web::get().to(ws_index)))
    })
    .bind("127.0.0.1:9001")?
    .run()
    .await
}
