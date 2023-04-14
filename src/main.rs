use std::string;

use actix_web::{App, HttpServer, middleware::Logger, web::Bytes};
use awc;
use actix_files as fs;
use redis::{Connection, RedisError, RedisResult};
use serde::{Deserialize};
use log::{info};

//Load Data into Redis Database

//query for redis database by location 

//increment downage reports

#[derive(Deserialize)]
struct json_format {
    id: String,
    url: String,
    commonName: String,
    placeType: String,
    additionalProperties: String,
    lat: f64,
    lon: f64,
}

const api_ping_time_secs: u64 = 10;

async fn api_request() -> Result<Bytes, Box<dyn std::error::Error>>{
    //make http request and handle response with api
    let client = awc::Client::default();
    let req = client.get("https://api.tfl.gov.uk/BikePoint/");
    let mut res = req.send().await?;
    Ok(res.body().limit(2500000).await?)
}   

async fn deserialize(data: Bytes) -> Result<json_format, Box<dyn std::error::Error>> {
    let json_data: json_format = serde_json::from_slice(&data)?;
    Ok(json_data)
}

async fn do_i_update(server: &mut Connection) -> Result<bool, Box<dyn std::error::Error>> {
    let res: u64 = redis::cmd("GET").arg("api_timestamp").query(server)?;
    if res - std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs() > api_ping_time_secs {
        return Ok(true);
    }
    Ok(false)
}

async fn update_redis(server: &mut Connection, data: Vec::<json_format>) -> Result<bool, Box<dyn std::error::Error>> {
    let _: () = redis::cmd("SET").arg("api_timestamp").arg(std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs()).query(server)?;
    let checksums: Vec<Result<redis::Value, RedisError>> = data.iter().map(|x| 
         redis::pipe()
        .cmd("SET").arg(&x.id).arg(&x.url)
        .cmd("SET").arg(&x.id).arg(&x.commonName)
        .cmd("SET").arg(&x.id).arg(&x.placeType)
        .cmd("JSON.SET").arg(&x.id).arg("$").arg(&x.additionalProperties)
        .cmd("SET").arg(&x.id).arg(&x.lat)
        .cmd("SET").arg(&x.id).arg(&x.lon)
        .query(server)
    ).collect();
    
    Ok(true)
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //start the logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));


    //Create a server for each thread
    HttpServer::new(|| 
        //Check if data is too old

        //if too old run api request and update redis

        //create the httpserver details with the app builder
        App::new()
        .wrap(Logger::default())
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
