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
    println!("{:?}", r);
    let room_manager_addr = room_manager.get_ref().clone();
    let res = ws::start(ClientActor::new(room_manager_addr), &r, stream);
    println!("{:?}", res);
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
