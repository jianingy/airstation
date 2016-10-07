// Jianing Yang <jianingy.yang@gmail.com> @  4 Oct, 2016

use chrono::naive::datetime::NaiveDateTime;
use r2d2;
use r2d2_mysql::MysqlConnectionManager;

use errors::*;
use models::AirQuality;

pub type Pool = r2d2::Pool<MysqlConnectionManager>;
type Connection = r2d2::PooledConnection<MysqlConnectionManager>;

pub fn init_db(url: &str) -> Result<Pool> {
    let config = r2d2::Config::default();
    let manager = try!(
        MysqlConnectionManager::new(url)
            .chain_err(|| db_error!("cannot connect to database: {}", url))
    );
    r2d2::Pool::new(config, manager)
        .chain_err(|| db_error!("cannot create connection pool"))
}

pub fn init_table(db: &mut Connection) -> Result<()> {
    let mut stmt = try!(
        db.prepare("CREATE TABLE IF NOT EXISTS air_quality ( \
                    id SERIAL PRIMARY KEY, \
                    pm10_cf1 INT, \
                    pm25_cf1 INT, \
                    pm100_cf1 INT, \
                    pm10 INT, \
                    pm25 INT, \
                    pm100 INT, \
                    cm3 INT, \
                    cm5 INT, \
                    cm10 INT, \
                    cm25 INT, \
                    cm50 INT, \
                    cm100 INT, \
                    created_at TIMESTAMP, \
                    UNIQUE KEY `created_at` (`created_at`) )")
            .chain_err(|| db_error!("cannot create table"))
    );
    try!(stmt.execute(()).chain_err(|| db_error!("cannot create table")));
    Ok(())
}

pub fn add_air_quality(db: &mut Connection, v: &AirQuality) -> Result<()> {
    let mut stmt = try!(
        db.prepare("INSERT INTO air_quality ( \
                    pm10_cf1, pm25_cf1, pm100_cf1, \
                    pm10, pm25, pm100, \
                    cm3, cm5, cm10, cm25, cm50, cm100) \
                    VALUES(?,?,?,?,?,?,?,?,?,?,?,?)")
            .chain_err(|| db_error!("cannot create table"))
    );
    try!(stmt.execute((
        &v.pm10_cf1,
        &v.pm25_cf1,
        &v.pm100_cf1,
        &v.pm10,
        &v.pm25,
        &v.pm100,
        &v.cm3,
        &v.cm10,
        &v.cm5,
        &v.cm25,
        &v.cm50,
        &v.cm100
    )).chain_err(|| db_error!("cannot insert data")));
    Ok(())
}

pub fn get_air_quality(db: &mut Connection, start: NaiveDateTime, end: NaiveDateTime, interval: u32)
                       -> Result<Vec<AirQuality>> {
    let mut stmt = try!(
        db.prepare("SELECT \
                    FROM_UNIXTIME(FLOOR(UNIX_TIMESTAMP(created_at) / ?) * ?) AS created_at, \
                    ROUND(AVG(pm10_cf1)) AS pm10_cf1, \
                    ROUND(AVG(pm25_cf1)) AS pm25_cf1, \
                    ROUND(AVG(pm100_cf1)) AS pm100_cf1, \
                    ROUND(AVG(pm10)) AS pm10, \
                    ROUND(AVG(pm25)) AS pm25, \
                    ROUND(AVG(pm100)) AS pm100, \
                    ROUND(AVG(cm3)) AS cm3, \
                    ROUND(AVG(cm5)) AS cm5, \
                    ROUND(AVG(cm10)) AS cm10, \
                    ROUND(AVG(cm25)) AS cm25, \
                    ROUND(AVG(cm50)) AS cm50, \
                    ROUND(AVG(cm100)) AS cm100
                    FROM air_quality \
                    WHERE created_at > ? AND created_at < ? \
                    GROUP BY (UNIX_TIMESTAMP(created_at) DIV ?)
                  ORDER BY created_at ASC")
            .chain_err(|| db_error!("SQL prepare error"))
    );
    let mut records = try!(stmt.execute((interval, interval, start, end, interval))
                           .chain_err(|| db_error!("SQL statement error")));
    let mut vec: Vec<AirQuality> = Vec::new();
    for record in records {
        if let Ok(mut row) = record {
            let record =AirQuality {
                pm10_cf1: row.take("pm10_cf1").unwrap(),
                pm25_cf1: row.take("pm25_cf1").unwrap(),
                pm100_cf1: row.take("pm100_cf1").unwrap(),
                pm10: row.take("pm10").unwrap(),
                pm25: row.take("pm25").unwrap(),
                pm100: row.take("pm100").unwrap(),
                cm3: row.take("cm3").unwrap(),
                cm5: row.take("cm5").unwrap(),
                cm10: row.take("cm10").unwrap(),
                cm25: row.take("cm25").unwrap(),
                cm50: row.take("cm50").unwrap(),
                cm100: row.take("cm100").unwrap(),
                created_at: row.take("created_at").unwrap()

            };
            vec.push(record)
        }
    };
    Ok(vec)
}
