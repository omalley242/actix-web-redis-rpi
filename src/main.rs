use actix_web::{App, HttpServer, HttpRequest, middleware::Logger, web::{Bytes}, get, HttpResponse, web};
use awc;
use actix_files as fs;
use redis::{Connection};
use serde::{Deserialize, Serialize};
use log::{info};
use std::str;
//Load Data into Redis Database

//query for redis database by location 

//increment downage reports

#[derive(Serialize, Deserialize)]
struct JsonFormat {
    id: String,
    url: String,
    commonName: String,
    placeType: String,
    additionalProperties: Vec::<SubJsonFormat>,
    lat: f64,
    lon: f64,
}

#[derive(Serialize, Deserialize)]
struct SubJsonFormat {
    catergory: String,
    key: String, 
    sourceSystemKey: String,
    value: String,
    modified: String,
}

const API_PING_TIME_SECS: u64 = 10;

async fn api_request() -> Result<Bytes, Box<dyn std::error::Error>>{
    info!("requesting tfl api data");
    let client = awc::Client::default();
    let req = client.get("https://api.tfl.gov.uk/BikePoint/");
    let mut res = req.send().await?;
    Ok(res.body().limit(2500000).await?)
}   

async fn deserialize(data: Bytes) -> Result<Vec::<JsonFormat>, Box<dyn std::error::Error>> {
    info!("deserializing json data");
    let JsonData: Vec::<JsonFormat> = serde_json::from_slice(&data)?;
    Ok(JsonData)
}

async fn do_i_update(server: &mut Connection) -> Result<bool, Box<dyn std::error::Error>> {
    info!("checking time from redis");
    info!("current time in secs: {}", std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs());
    let res: u64 = redis::cmd("GET").arg("api_timestamp").query(server)?;
    info!("{}", res);
    if std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs() - res > API_PING_TIME_SECS {
        return Ok(true);
    }
    Ok(false)
}

async fn update_redis(server: &mut Connection, data: Vec::<JsonFormat>) -> Result<bool, Box<dyn std::error::Error>> {
    info!("updating redis json data");
    let _: () = redis::cmd("SET").arg("api_timestamp").arg(std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs()).query(server)?;

    for x in data {
        redis::cmd("JSON.SET").arg(&x.id).arg("$").arg(serde_json::to_string(&x)?).query(server)?;
    }
    Ok(true)
}

async fn poll_update() -> Result<HttpResponse, Box<dyn std::error::Error>> {
    info!("updating redis data");
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut server = client.get_connection()?;
    let update = do_i_update(&mut server).await?;
    if update {
        let data = api_request().await?;
        let json_data = deserialize(data).await?;
        update_redis(&mut server, json_data).await?;
    }
    Ok(HttpResponse::Ok().finish())
}

#[get("/query/{id}")]
async fn redis_query(req: HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    poll_update().await?;
    info!("querying the data");
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut server = client.get_connection()?;
    let query_id = req.match_info().get("id").unwrap().to_string();
    let res: String = redis::cmd("JSON.GET").arg(query_id).arg("$").query(&mut server)?;
    Ok(HttpResponse::Ok().body(res))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //start the logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    //Create a server for each thread
    HttpServer::new(|| 
        App::new()
        .wrap(Logger::default())
        .route("/test", web::get().to(poll_update))
        .service(redis_query)
        .service(fs::Files::new("/", "./static/home_page")
            .show_files_listing()
            .index_file("home_page.html")
            .use_last_modified(true),
        )
    )
    //bind it to the open port on 8080 (http not ssl)
    .bind(("0.0.0.0", 8080))?
    //start the server
    .run()
    .await
}
