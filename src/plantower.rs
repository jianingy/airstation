// Jianing Yang <jianingy.yang@gmail.com> @  3 Oct, 2016
use chrono::offset::local::Local;
use serial;
use serial::posix::TTYPort;
use serial::prelude::*;
use std::io::prelude::*;
use std::time::Duration;

use errors::*;
use models::AirQuality;

#[derive(PartialEq)]
pub enum DeviceType {
    PMS5003
}

pub struct PlanTower {
    uart: TTYPort
}

impl PlanTower {

    pub fn init(device: &str, device_type: DeviceType) -> Result<PlanTower> {
        // Only PMS5003 supported currently
        assert!(device_type == DeviceType::PMS5003);

        let settings = serial::PortSettings {
            baud_rate:    serial::Baud9600,
            char_size:    serial::Bits8,
            parity:       serial::ParityNone,
            stop_bits:    serial::Stop1,
            flow_control: serial::FlowNone
        };
        let timeout = Duration::from_secs(1);
        let mut uart = try!(
            serial::open(&device)
                .chain_err(|| uart_error!("cannot open uart {}", device)));
        try!(uart.configure(&settings)
             .chain_err(|| uart_error!("cannot configure uart {}", device)));
        try!(uart.set_timeout(timeout)
             .chain_err(|| uart_error!("cannot set timeout for {}", device)));

        // drain buffer
        Ok(PlanTower {
            uart: uart
        })
    }

    pub fn read(&mut self) -> Result<AirQuality> {
        let bs = try!(self.read_uart());
        self.translate(&bs)
    }

    fn read_uart(&mut self) -> Result<Vec<u8>> {
        // read and verify if header starts with 'BM'
        let mut hd = [0; 2];
        loop {
            try!(self.uart.read_exact(&mut hd[0..1])
                 .chain_err(|| uart_error!("cannot read")));
            if hd[0] != 0x42 {
                continue
            }
            try!(self.uart.read_exact(&mut hd[1..])
                 .chain_err(|| uart_error!("cannot read")));
            if hd[1] != 0x4d {
                continue
            }
            break;
        }
        // get data length
        let mut sz: Vec<u8> = vec![0; 2];
        try!(self.uart.read_exact(&mut sz)
             .chain_err(|| uart_error!("cannot read")));
        let len = ((sz[0] as usize) << 8) + sz[1] as usize;
        let mut bs: Vec<u8> = vec![0;len];

        // read and check if signature matches
        try!(self.uart.read_exact(&mut bs)
             .chain_err(|| uart_error!("cannot read")));
        let &(data, sign) = &bs.split_at(len - 2);
        let checksum = hd.iter().map(|x| *x as u16).sum::<u16>()
            + sz.iter().map(|x| *x as u16).sum::<u16>()
            + data.iter().map(|x| *x as u16).sum::<u16>();
        let sign = ((sign[0] as u16) << 8) + sign[1] as u16;
        if checksum != sign {
            return Err(data_error!("[plantower] signautre mismatched").into())
        }

        // return only data part
        Ok(Vec::from(data))
    }

    fn translate(&self, bs: &[u8]) -> Result <AirQuality> {
        let vals = bs.chunks(2)
            .map(|b| ((b[0] as u16) << 8) + b[1] as u16)
            .collect::<Vec<u16>>();;
        Ok(AirQuality {
            pm10_cf1: vals[0],
            pm25_cf1: vals[1],
            pm100_cf1: vals[2],
            pm10: vals[3],
            pm25: vals[4],
            pm100: vals[5],
            cm3: vals[6],
            cm5: vals[7],
            cm10: vals[8],
            cm25: vals[9],
            cm50: vals[10],
            cm100: vals[11],
            created_at: Local::now().naive_local()
        })
    }
}
