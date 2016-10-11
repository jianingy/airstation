// Jianing Yang <jianingy.yang@gmail.com> @  4 Oct, 2016
use serde_json::value::{ToJson, Value};
use std::collections::BTreeMap;
use chrono::naive::datetime::NaiveDateTime;

#[derive(Debug)]
pub struct AirQuality {
    pub pm10_cf1: u16,
    pub pm25_cf1: u16,
    pub pm100_cf1: u16,
    pub pm10: u16,
    pub pm25: u16,
    pub pm100: u16,
    pub cm3: u16,
    pub cm5: u16,
    pub cm10: u16,
    pub cm25: u16,
    pub cm50: u16,
    pub cm100: u16,
    pub created_at: NaiveDateTime,
}


impl ToJson for AirQuality {
    fn to_json(&self) -> Value {
        let mut map = BTreeMap::new();
        map.insert("PM10_CF1".to_string(), self.pm10_cf1.to_json());
        map.insert("PM25_CF1".to_string(), self.pm25_cf1.to_json());
        map.insert("PM100_CF1".to_string(), self.pm100_cf1.to_json());
        map.insert("PM10".to_string(), self.pm100.to_json());
        map.insert("PM25".to_string(), self.pm25.to_json());
        map.insert("PM100".to_string(), self.pm100.to_json());
        map.insert("created_at".to_string(), self.created_at.to_json());
        Value::Object(map)
    }
}

#[derive(Debug)]
pub struct Environment {
    pub humidity: i32,
    pub temperature: i32,
    pub created_at: NaiveDateTime,
}

impl ToJson for Environment {
    fn to_json(&self) -> Value {
        let mut map = BTreeMap::new();
        map.insert("humidity".to_string(), self.humidity.to_json());
        map.insert("temperature".to_string(), self.temperature.to_json());
        map.insert("created_at".to_string(), self.created_at.to_json());
        Value::Object(map)
    }
}
