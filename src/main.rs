extern crate redis;
use actix_web::{web, App, HttpResponse, HttpServer, get, middleware::Logger};
use awc::Client;
use redis::{Commands, RedisError};

#[get("/query/{station_name}")]
async fn query(path: web::Path<(u32,)>) -> HttpResponse{
    print!("{:?}", path);
    print!("{}", fetch_an_integer().unwrap());
    HttpResponse::Ok().body("")
}


fn fetch_an_integer() -> redis::RedisResult<isize> {
    // connect to redis
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;
    // throw away the result, just make sure it does not fail
    let _ : () = con.set("my_key", 42)?;
    // read back the key and return it.  Because the return value
    // from the function is a result for integer this will automatically
    // convert into one.
    con.get("my_key")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(|| 
        App::new()
        .wrap(Logger::default())
        .route("/", web::get().to(HttpResponse::Ok)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
