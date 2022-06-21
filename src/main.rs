use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use gtmpl::{self, Value};
use log::*;
use std::collections::HashMap;
use std::env::set_current_dir;
use std::path::PathBuf;
use std::process;
use structopt::StructOpt;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    #[structopt(
        name = "TEMPLATE_DIR",
        parse(from_os_str),
        env = "TPLSRV_DIR",
        default_value = "."
    )]
    dir: PathBuf,

    #[structopt(short, long, default_value = "8080")]
    port: u16,

    #[structopt(short, long, default_value = "0.0.0.0")]
    listen: String,
}

#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/{template}")]
async fn template_service(
    template: web::Path<String>,
    query: web::Query<HashMap<String, String>>,
) -> Result<impl Responder, Box<dyn std::error::Error>> {
    let filename = format!("{}.tmpl", template);
    let mut file = match File::open(&filename).await {
        Ok(x) => x,
        Err(_) => return Ok(None),
    };

    let mut buffer = vec![];
    file.read_to_end(&mut buffer).await?;
    let template = String::from_utf8(buffer)?;

    let query = query.into_inner();
    let context = Value::from(query.clone());
    let result = gtmpl::template(&template, context)?;
    info!("{} rendered with: {:?}", filename, query);
    Ok(Some(HttpResponse::Ok().body(result)))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();
    pretty_env_logger::init();

    if let Err(e) = set_current_dir(&opt.dir) {
        eprintln!("{}: {}", opt.dir.display(), e);
        process::exit(1);
    }

    HttpServer::new(|| App::new().service(healthz).service(template_service))
        .bind((opt.listen, opt.port))?
        .run()
        .await
}
