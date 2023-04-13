use actix_web::{App, HttpServer, middleware::Logger};
use awc;
use actix_files as fs;
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
    additionalProperties: Vec::<serde_json::Value>,
    lat: f64,
    lon: f64,
}

async fn tfl_api_request_handler() -> Result<String, Box<dyn std::error::Error>>{
    //make http request and handle response with api
    let client = awc::Client::default();
    let req = client.get("https://api.tfl.gov.uk/BikePoint/");
    let mut res = req.send().await?;
    let body = res.body().limit(2500000).await?;
    //Deserialize ----
    let value_enum: serde_json::Value = serde_json::from_slice(&body)?;
    let json_stations_list = value_enum.as_array().unwrap();
    for station in json_stations_list{
        let Data: json_format = serde_json::from_value(station.clone())?;
        //Load Deserialized data into Redis
        info!("{} : {}", Data.commonName, Data.additionalProperties[2]["value"]);
    }
    Ok("Fetched the tfl station data".to_string())
}   

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //start the logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("{}", tfl_api_request_handler().await.unwrap());

    //Create a server for each thread
    HttpServer::new(|| 
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
