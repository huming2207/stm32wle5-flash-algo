#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use cortex_m::interrupt::CriticalSection;
use flash_algorithm::*;
use rtt_target::{rprintln, rtt_init_print};
use stm32wlxx_hal::{pac::Peripherals, rcc, flash::{Flash, Page, AlignedAddr}};

struct Algorithm {
    dp: Peripherals
}

algorithm!(Algorithm, {
    flash_address: 0x08000000,
    flash_size: 0x40000,
    page_size: 0x800,
    empty_value: 0xff,
    sectors: [{
        size: 0x800,
        address: 0,
    }]
});

impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        #[cfg(debug_assertions)] {
            rtt_init_print!();
            rprintln!("Init");
        }

        let mut dp = Peripherals::take().unwrap();
        let cs = unsafe { &CriticalSection::new() };
        unsafe {
            rcc::set_sysclk_msi(
                &mut dp.FLASH,
                &mut dp.PWR,
                &mut dp.RCC,
                rcc::MsiRange::Range16M, // Maybe we should consider 16MHz here so that no flash wait cycle needed??
                cs,
            );
        }

        Ok(Algorithm { dp })
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        #[cfg(debug_assertions)]
        rprintln!("Erase All");

        let mut flash = Flash::unlock(&mut self.dp.FLASH);
        let ret = unsafe { flash.mass_erase() };
        match ret {
            Ok(_) => return Ok(()),
            Err(_) => return Err(ErrorCode::new(0x70d0).unwrap())
        }
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        #[cfg(debug_assertions)]
        rprintln!("Erase sector addr:0x{:04x}", addr);

        let mut flash = Flash::unlock(&mut self.dp.FLASH);
        let page = match Page::from_addr(addr as usize) {
            Some(page) => page,
            None => {
                return Err(ErrorCode::new(1).unwrap());
            }
        };

        let ret = unsafe { flash.page_erase(page) };
        match ret {
            Ok(_) => return Ok(()),
            Err(_) => {
                return Err(ErrorCode::new(2).unwrap())
            }
        }
    }

    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {
        #[cfg(debug_assertions)]
        rprintln!("Program Page addr:{} size:{}", addr, data.len());

        let mut flash = Flash::unlock(&mut self.dp.FLASH);
        let addr = match AlignedAddr::try_from(addr) {
            Ok(addr) => addr,
            Err(_) => return Err(ErrorCode::new(1).unwrap())
        };

        let ret = unsafe { flash.program_bytes(data, addr) };
        match ret {
            Ok(_) => return Ok(()),
            Err(_) => return Err(ErrorCode::new(2).unwrap())
        }
    }
}

impl Drop for Algorithm {
    fn drop(&mut self) {
        // TODO: Add code here to uninitialize the flash algorithm.
    }
}
