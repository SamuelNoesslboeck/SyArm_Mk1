use std::sync::{Arc, Mutex};

use actix::{Actor, StreamHandler};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use actix_web_actors::ws;

use syarm_lib::{SyArm, SyArmResult};
use syarm_lib::{Interpreter, init_interpreter};

struct AppData {
    pub intpr : Interpreter<SyArm, SyArmResult<()>>
}

struct MyWs {
    pub data : Arc<Mutex<AppData>>
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => { 
                let mut data = self.data.lock().unwrap();
                data.intpr.interpret(&text);
                ctx.text(text)
            },
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

#[get("/cons")]
async fn cons(data_mutex : web::Data<Mutex<AppData>>) -> impl Responder {
    let data = data_mutex.lock().unwrap();
    HttpResponse::Ok().content_type("application/json").body(data.intpr.mach.get_cons_str_pretty())
}

#[get("/intpr")]
async fn intpr(data_mutex : web::Data<Mutex<AppData>>, req: HttpRequest, stream: web::Payload) -> impl Responder {
    ws::start(MyWs {
        data: data_mutex.into_inner().clone()
    }, &req, stream)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(Mutex::new(AppData {
                intpr: init_interpreter(SyArm::load_json("res/syarm_const.json"))
            })))
            .service(cons)
            .service(intpr)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}