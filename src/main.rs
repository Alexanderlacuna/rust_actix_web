
use actix_web::{Error,HttpRequest,web,App,HttpResponse,HttpServer,Responder,get,Result};
use serde::{Deserialize,Serialize};

use  listenfd::ListenFd;
use std::sync::Mutex;
use futures::future::{ready,Ready};


#[derive(Serialize)]
struct MyObj {
    name:&'static str

}

impl Responder for MyObj {
    type Error=Error;
    type Future=Ready<Result<HttpResponse,Error>>;
    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        let body=serde_json::to_string(&self).unwrap();
        // create response type and set content-type
        ready(Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(body)
    ))
        
    }
}
struct AppStateWithCounter {
    counter:Mutex<i32>,
}

fn config(cfg:&mut web::ServiceConfig){
    cfg.service(
        web::resource("/app")
        .route(web::get().to(|| HttpResponse::Ok().body("app")))
        .route(web::head().to(|| HttpResponse::MethodNotAllowed())),
    );
}
fn scope_config(cfg:&mut web::ServiceConfig){
    cfg.service(
        web::resource("/test")
        .route(web::get().to(|| HttpResponse::Ok().body("test")))
        .route(web::head().to(|| HttpResponse::MethodNotAllowed())),
    );
}

struct AppState {
    app_name:String
}



async fn start(data:web::Data<AppStateWithCounter>)->String{
    let mut counter=data.counter.lock().unwrap();
    *counter+=1;
    format!("Requests number {}",counter)

}

// json deserialization
// deserialize item from request body

#[derive(Deserialize)]
struct Info3{
    username:String,

    email:String,

    password:String,
    
}
async fn json_extraction(info:web::Json<Info3>)->Result<String>{
Ok(format!("welcome {} with email {} and password {}",info.username,info.email,info.password))
}


#[derive(Deserialize)]
struct Info2{
    username:String,
}

// query extraction
async fn query_extraction(info:web::Query<Info2>)->String{
    format!("Welcome {}",info.username)
}

// get or query the request for path parameters by name
async fn query_by_name(req:HttpRequest)->Result<String>{
    let name:String=req.match_info().get("friend").unwrap().parse().unwrap();
    let userid:i32=req.match_info().query("userid").parse().unwrap();
    Ok(format!("Welcome {} with id {}",name,userid))

}

async fn extraction(info:web::Path<(u32,String)>)->Result<String>{
    Ok(format!("Welcome {} userid {}",info.1,info.0))

}
#[derive(Deserialize)]
struct Info {
    userid:u32, 
    friend:String
}
async fn extract_to_type(info:web::Path<Info>)->Result<String>{

    Ok(format!("Welcome {}, userid {}!", info.friend, info.userid))

}
async fn index(data: web::Data<AppState>)->String{
    let app_name=&data.app_name;
    format!("Hello {}",app_name)
}

async fn index2()->impl Responder{

    HttpResponse::Ok().body("hello world again")

}

async fn serializable()->impl Responder {
    MyObj {
        name:"user"
    }
}
#[get("/hello3")]
async fn index3()->impl Responder{
    HttpResponse::Ok().body("Hey there!")
}

#[actix_rt::main]
async fn main()->std::io::Result<()>{
    let counter=web::Data::new(AppStateWithCounter{
        counter:Mutex::new(0),
    });
    let mut listenfd=ListenFd::from_env();
    let mut server=HttpServer::new(move|| {
        App::new()

          .configure(config)
          .service(web::scope("/api").configure(scope_config))

          .app_data(counter.clone())

             .data(AppState {
                 app_name:String::from("actix web")
             })

             .service(
                  web::scope("/users")
                 .route("/again",web::get().to(index2))
                 .route("/start",web::get().to(start))
                 .route("/serializable",web::get().to(serializable))
             )
            .route("/",web::get().to(index))
            .route("/user2/{userid}/{friend}", web::get().to(extract_to_type))
            .route("/queries", web::get().to(query_extraction))
            .route("/json", web::post().to(json_extraction))
            
                 
            .service(index3)

    });
    server=if let Some(l)=listenfd.take_tcp_listener(0).unwrap(){
        server.listen(l)?
    }else{
        server.bind("127.0.0.1:3000")?
    };

    server.run().await
    // .bind("127.0.0.1:8081")?
    // .run()
    // .await

}