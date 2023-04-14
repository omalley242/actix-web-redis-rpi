use actix_web::{App, HttpServer, middleware::Logger, web::Bytes, get, HttpResponse, web};
use awc;
use actix_files as fs;
use redis::{Connection, RedisError};
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

const API_PING_TIME_SECS: u64 = 10;

async fn api_request() -> Result<Bytes, Box<dyn std::error::Error>>{
    info!("requesting tfl api data");
    let client = awc::Client::default();
    let req = client.get("https://api.tfl.gov.uk/BikePoint/");
    let mut res = req.send().await?;
    Ok(res.body().limit(2500000).await?)
}   

async fn deserialize(data: Bytes) -> Result<Vec::<json_format>, Box<dyn std::error::Error>> {
    info!("deserializing json data");
    let json_data: Vec::<json_format> = serde_json::from_slice(&data)?;
    Ok(json_data)
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

async fn update_redis(server: &mut Connection, data: Vec::<json_format>) -> Result<bool, Box<dyn std::error::Error>> {
    info!("updating redis json data");
    let _: () = redis::cmd("SET").arg("api_timestamp").arg(std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs()).query(server)?;
    let _: Vec<Result<redis::Value, RedisError>> = data.iter().map(|x| 
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

async fn test_func() -> Result<HttpResponse, Box<dyn std::error::Error>> {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //start the logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    //Create a server for each thread
    HttpServer::new(|| 
        App::new()
        .wrap(Logger::default())
        .route("/test", web::get().to(test_func))
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
