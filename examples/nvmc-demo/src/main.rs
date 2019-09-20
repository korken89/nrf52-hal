#![no_std]
#![no_main]

use cortex_m_rt::entry;
use nrf52832_hal::nvmc::{Nvmc, Error};
use panic_semihosting as _;

#[entry]
fn main() -> ! {
    let p = nrf52832_hal::nrf52832_pac::Peripherals::take().unwrap();

    let mut nvmc = Nvmc::new(p.NVMC);

    let mut flash_page = nvmc.erase_page(32).unwrap();

    let buf = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
    let buf1 = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let buf2 = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
    let buf3 = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18];

    assert_eq!(flash_page.write_block(0, buf), Ok(buf.len()));
    assert_eq!(slice_from_address(0x20000 + 0x200 * 0, buf.len()), buf);

    assert_eq!(flash_page.write_block(1, buf1), Ok(buf1.len()));
    assert_eq!(slice_from_address(0x20000 + 0x200 * 1, buf.len()), buf);

    assert_eq!(flash_page.write_block(2, buf2), Ok(buf2.len()));
    assert_eq!(slice_from_address(0x20000 + 0x200 * 2, buf.len()), buf);

    assert_eq!(flash_page.write_block(3, buf3), Ok(buf3.len()));
    assert_eq!(slice_from_address(0x20000 + 0x200 * 3, buf.len()), buf);

    assert_eq!(flash_page.write_block(4, buf), Ok(buf.len()));
    assert_eq!(slice_from_address(0x20000 + 0x200 * 4, buf.len()), buf);

    assert_eq!(flash_page.write_block(5, buf), Ok(buf.len()));
    assert_eq!(slice_from_address(0x20000 + 0x200 * 5, buf.len()), buf);

    assert_eq!(flash_page.write_block(6, buf), Ok(buf.len()));
    assert_eq!(slice_from_address(0x20000 + 0x200 * 6, buf.len()), buf);

    assert_eq!(flash_page.write_block(7, buf), Ok(buf.len()));
    assert_eq!(slice_from_address(0x20000 + 0x200 * 7, buf.len()), buf);

    assert_eq!(flash_page.write_block(8, buf), Err(Error::OutOfBounds));

    loop {
        continue;
    }
}

fn slice_from_address(addr: usize, size: usize) -> &'static [u8] {
    unsafe { core::slice::from_raw_parts(addr as *const u8, size) }
}
