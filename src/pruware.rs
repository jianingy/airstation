// Jianing Yang <jianingy.yang@gmail.com> @ 11 Oct, 2016

use prusst::{Pruss, IntcConfig, Evtout, Sysevt, Error as PrusstError};
use std::fs::File;
use errors::*;

const BASE: usize = 0x12000;

pub fn create_pruss() -> Result<Pruss<'static>> {
    match Pruss::new(&IntcConfig::new_populated()) {
        Ok(p) => Ok(p),
        Err(e) => Err((match e {
            PrusstError::AlreadyInstantiated
                => pru_error!("You can't instantiate more than one `Pruss` object at a time."),
            PrusstError::PermissionDenied
                => pru_error!("You do not have permission to access the PRU subsystem: \
                               maybe you should log as root?"),
            PrusstError::DeviceNotFound
                => pru_error!("The PRU subsystem could not be found: are you sure the `uio_pruss` \
                               module is loaded and supported by your kernel?"),
            PrusstError::OtherDeviceError
                => pru_error!("An unidentified problem occured with the PRU subsystem: \
                               do you have a valid overlay loaded?")
        }).into())
    }
}

pub fn read_from_dht11(mut pruss: Pruss<'static>) -> Option<(i32, i32)> {
    let (_, mut bank) = pruss.dram2.split_at(BASE);
    let bs = unsafe { bank.alloc_uninitialized::<[u8; 8]>() };
    let done_irq = pruss.intc.register_irq(Evtout::E0);
    let mut firmware = File::open("dht11.bin").unwrap();
    unsafe {
        pruss.pru0.load_code(&mut firmware).unwrap().run();
    }
    done_irq.wait();
    pruss.intc.clear_sysevt(Sysevt::S19);
    let humidity = (bs[3] as i32) * 100 + (bs[2] as i32);
    let celsius = (bs[1] as i32) * 100 + (bs[0] as i32);
    let sum = bs.iter().take(4).map(|x| *x as u16).sum::<u16>() & 0xff;
    debug!("DHT11 checksum = {:?}, bytes = {:?}", sum, bs);
    if (sum as u8) == bs[4] {
        Some((humidity, celsius))
    } else {
        None
    }
}

pub fn read_from_dht22(mut pruss: Pruss<'static>) -> Option<(i32, i32)> {
    let (_, mut bank) = pruss.dram2.split_at(BASE);
    let bs = unsafe { bank.alloc_uninitialized::<[u8; 8]>() };
    let done_irq = pruss.intc.register_irq(Evtout::E0);
    let mut firmware = File::open("dht11.bin").unwrap();
    unsafe {
        pruss.pru0.load_code(&mut firmware).unwrap().run();
    }
    done_irq.wait();
    pruss.intc.clear_sysevt(Sysevt::S19);
    let humidity = (((bs[3] as i32) << 8) + (bs[2] as i32)) * 10;
    let celsius = (((bs[1] as i32) << 8) + (bs[0] as i32)) * 10;
    let sum = bs.iter().take(4).map(|x| *x as u16).sum::<u16>() & 0xff;
    debug!("DHT22 checksum = {:?}, bytes = {:?}", sum, bs);
    if (sum as u8) == bs[4] {
        Some((humidity, celsius))
    } else {
        None
    }
}
