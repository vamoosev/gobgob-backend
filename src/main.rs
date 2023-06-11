use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, http::ConnectionType};
use rusqlite::{Connection, Result};

use dotenv::dotenv;

// ? should we use this or db?

// #[actix_web::post("/supersalainen/{filename}")]
// async fn post_file(filename: web::Path<String>, body: web::Bytes) -> std::io::Result<HttpResponse> {
//     let filterred = &filename
//         .into_inner()
//         .chars()
//         .filter(|&c| c != '/' && c != '.')
//         .collect::<String>();
//     let mut file = fs::File::create("./files/".to_string() + filterred)?;
//     file.write_all(&body).unwrap();

//     // Return a success response
//     Ok(HttpResponse::Ok().body(body))
// }
use tokio::sync::Mutex;
use std::sync::Arc;

#[actix_web::post("/supersalainen/json")]
async fn postjuttu(db: web::Data<Arc<Mutex<Connection>>>, body: String) -> std::io::Result<HttpResponse> {
    let db = db.lock().await;
    // turhaa kikkailua, mutta en keksinyt miten saada rusqlite::Connection
    // toimimaan ilman clonea
    // HACK: onko tämä turhaa kikkailua?
    //
    // make body into json    
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let mac = json["mac"].as_str().unwrap();
    let date = json["date"].as_str().unwrap();
    let computer = json["computer"].as_str().unwrap();
    db.execute(
        "INSERT INTO macs (mac, date, computer) VALUES (?1, ?2, ?3)",
        [mac, date, computer],
    ).unwrap();

    Ok(HttpResponse::Ok().body(body))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // create db connection

    let db = Arc::new(Mutex::new(Connection::open("test.db").unwrap()));
    db.lock().await.execute(
        "CREATE TABLE IF NOT EXISTS macs (
                mac TEXT NOT NULL,
                date TEXT NOT NULL,
                computer TEXT NOT NULL
            )",
        (),
    ).unwrap();
    
    dotenv().ok();

    // pass db to app
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(db.clone()))
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(postjuttu)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
