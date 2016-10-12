#[macro_use] extern crate error_chain;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate nickel;

extern crate chrono;
extern crate clap;
extern crate env_logger;
extern crate prusst;
extern crate r2d2;
extern crate r2d2_mysql;
extern crate serde_json;
extern crate serial;
extern crate sysfs_gpio;

#[macro_use] mod errors;
mod db_api;
mod models;
mod plantower;
mod pruware;

use chrono::duration::Duration;
use chrono::naive::datetime::NaiveDateTime;
use chrono::offset::local::Local;
use clap::{App, AppSettings, Arg, ArgMatches};
use env_logger::LogBuilder;
use log::{LogRecord, LogLevel, LogLevelFilter};
use nickel::{Nickel, QueryString, StaticFilesHandler};
use serde_json::value::{ToJson, Value};
use std::sync::Arc;
use std::thread;

use errors::*;
use plantower::PlanTower;

static VERSION: &'static str = "0.1.1";

lazy_static! {
    static ref OPTIONS: ArgMatches<'static> = {
        App::new("airstation")
            .version(VERSION)
            .about("an air quality monitorn")
            .global_setting(AppSettings::ColoredHelp)
            .arg(Arg::with_name("mysql")
                 .long("mysql")
                 .help("mysql connection string")
                 .required(true)
                 .takes_value(true))
            .arg(Arg::with_name("uart")
                 .long("uart")
                 .help("uart device for plantower sensor")
                 .required(true)
                 .takes_value(true))
            .arg(Arg::with_name("bind")
                 .long("bind")
                 .help("server listening address")
                 .required(true)
                 .default_value("0.0.0.0:8080")
                 .takes_value(true))
            .arg(Arg::with_name("verbose")
                 .short("v")
                 .multiple(true))
            .get_matches()
    };
}

fn init_logger() {
    let log_format = |record: &LogRecord| {
        let message = format!("[{}] {}",
                              match record.level() {
                                  LogLevel::Error => "!",
                                  LogLevel::Warn => "*",
                                  LogLevel::Info => "+",
                                  LogLevel::Debug => "#",
                                  LogLevel::Trace => "~",
                              },
                              record.args());
        message
    };
    let mut builder = LogBuilder::new();
    builder.format(log_format)
        .filter(None,
                match OPTIONS.occurrences_of("verbose") {
                    n if n > 2 => LogLevelFilter::Trace,
                    n if n == 2 => LogLevelFilter::Debug,
                    n if n == 1 => LogLevelFilter::Info,
                    _ => LogLevelFilter::Warn,
                });
    builder.init().unwrap();
}

fn plantower_run_once(port: &str) -> Result<models::AirQuality>{
    let mut device = try!(
        PlanTower::init(port, plantower::DeviceType::PMS5003)
    );
    debug!("uart port initialization ok");
    let data = try!(device.read());
    Ok(data)
}

fn start_plantower(db: Arc<db_api::Pool>) {
    thread::spawn(move || {
        let mut conn = db.get().unwrap();
        db_api::init_table(&mut conn).unwrap();
        debug!("plantower reader thread started.");
        loop {
            let mut conn = db.get().unwrap();
            match plantower_run_once(OPTIONS.value_of("uart").unwrap()) {
                Err(e) => {
                    info!("{:?}", e);
                    continue;
                }
                Ok(v) => {
                    info!("PlanTower Data: {:?}", v);
                    db_api::add_air_quality(&mut conn, &v).unwrap();
                }
            }
            thread::sleep(Duration::seconds(60).to_std().unwrap());
        }
    });
}

fn start_htreader(db: Arc<db_api::Pool>) {
    thread::spawn(move || {
        loop {
            let pruss = pruware::create_pruss();
            if let Some((humidity, celsius)) = pruware::read_from_dht22(pruss) {
                info!("DHT11 Data: humidity = {} / celsius = {}", humidity, celsius);
                let mut conn = db.get().unwrap();
                db_api::add_environment(&mut conn, humidity, celsius).unwrap();
                thread::sleep(Duration::seconds(60).to_std().unwrap());
            } else {
                warn!("error on read DHT11, will try again soon");
                thread::sleep(Duration::seconds(1).to_std().unwrap());
            }
        }
    });
}

fn start_dashboard(db: Arc<db_api::Pool>) {
    let mut server = Nickel::new();

    fn parse_range_argument(q: &nickel::Params) -> (NaiveDateTime, NaiveDateTime, u32)
    {
        let fmt = "%F_%T";
        let start = match q.get("start_date") {
            Some(x) => NaiveDateTime::parse_from_str(x, fmt).unwrap(),
            None => Local::now().naive_local() - Duration::hours(6)
        };
        let end = match q.get("end_date") {
            Some(x) => NaiveDateTime::parse_from_str(x, fmt).unwrap(),
            None => Local::now().naive_local()
        };
        let interval = match q.get("interval") {
            Some(x) => x.parse::<u32>().unwrap(),
            None => 900u32
        };

        (start, end, interval)
    }
    let (db1, db2) = (db.clone(), db.clone());
    server.utilize(StaticFilesHandler::new("static/"));
    server.utilize(router! {
        get "/api/v1/version" => |_req, _res| {
            "1.0.0"
        }
        get "/api/v1/air" => |req, _res| {
            let mut conn = db1.get().unwrap();
            let q = req.query();
            let (start, end, interval) = parse_range_argument(q);
            let records = db_api::get_air_quality(&mut conn, start, end, interval).unwrap();
            let v = records.iter().map(|x| x.to_json()).collect::<Vec<Value>>();
            serde_json::to_string(&v).unwrap()
        }
        get "/api/v1/environment" => |req, _res| {
            let mut conn = db2.get().unwrap();
            let q = req.query();
            let (start, end, interval) = parse_range_argument(q);
            let records = db_api::get_environment(&mut conn, start, end, interval).unwrap();
            let v = records.iter().map(|x| x.to_json()).collect::<Vec<Value>>();
            serde_json::to_string(&v).unwrap()
        }
    });
    server.listen(OPTIONS.value_of("bind").unwrap());
}

fn main() {
    init_logger();
    let db = Arc::new(db_api::init_db(OPTIONS.value_of("mysql").unwrap()).unwrap());
    start_plantower(db.clone());
    start_htreader(db.clone());
    // run dashboard server
    start_dashboard(db.clone());

}
