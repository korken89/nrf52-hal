//! HAL interface to the NVMC peripheral
//!
//! See product specification, chapter 11.

use crate::target::{nvmc, NVMC};
use core::{ops::Deref, ptr};

// As per Figure 4: Memory layout
const PAGE_SIZE_BYTES: usize = 0x1000;

// As per Figure 4: Memory layout
const BLOCK_SIZE_BYTES: usize = PAGE_SIZE_BYTES / 8;

/// The number of flash memory pages
#[cfg(feature = "52810")]
pub const NUMBER_OF_PAGES: usize = 48;
#[cfg(all(feature = "52832", not(feature = "SMALL_FLASH")))]
pub const NUMBER_OF_PAGES: usize = 128;
#[cfg(all(feature = "52832", feature = "SMALL_FLASH"))]
pub const NUMBER_OF_PAGES: usize = 64;
#[cfg(feature = "52840")]
pub const NUMBER_OF_PAGES: usize = 256;
#[cfg(feature = "9160")]
pub const NUMBER_OF_PAGES: usize = 256;

// As per Figure 4: Memory layout
const MAX_BLOCK_WRITES: usize = 181;

/// NVMC errors
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    /// An operation was out of bounds
    OutOfBounds,
    /// The maximum number of writes has been reached for a block
    MaxBlockWritesReached,
}

/// Interface to an NVMC instance
///
/// This is a very simple interface that comes with the following limitations:
/// - The instance may only operate on one flash page at a time, even though the underlying
///   peripheral allows for parallel access.
pub struct Nvmc<T>(T);

impl<T> Nvmc<T>
where
    T: Instance,

{
    /// Create a new `Nvmc` from an NVMC instance
    pub fn new(nvmc: T) -> Self {
        Self(nvmc)
    }

    /// Return the raw interface to the underlying NVMC peripheral
    pub fn free(self) -> T {
        self.0
    }

    /// Erases a page based on its page index, where `page_index` is limited to [0..NUMBER_OF_PAGES]
    pub fn erase_page(&mut self, page_index: usize) -> Result<ErasedFlashPage<T>, Error> {
        let flash_page_address = FlashPage::from_index(page_index)?.to_address();

        // Enable erase
        self.0.config.write(|w| w.wen().een());

        // Erase page
        unsafe {
            self.0
                .erasepage
                .write(|w| w.erasepage().bits(flash_page_address as u32));
        }

        // Wait for operation to complete
        while self.0.ready.read().ready().is_busy() {}

        // Disable erase
        self.0.config.write(|w| w.wen().ren());

        Ok(ErasedFlashPage::new(self, flash_page_address))
    }
}

impl<'a, T> ErasedFlashPage<'a, T>
where
    T: Instance,
{
    /// Hidden new function for internal use
    fn new(nvmc: &'a mut Nvmc<T>, page_start: *mut u32) -> Self {
        Self {
            blocks: [
                // Each block that we can write to
                FlashBlock::new(),
                FlashBlock::new(),
                FlashBlock::new(),
                FlashBlock::new(),
                FlashBlock::new(),
                FlashBlock::new(),
                FlashBlock::new(),
                FlashBlock::new(),
            ],
            page_start,
            nvmc,
        }
    }

    /// Write access to the flash, will write a buffer of up to 512 bytes to the current block ID.
    /// The return indicates how many bytes were written from the buf, i.e. `Ok(bytes_written)`
    pub fn write_block(&mut self, block_id: usize, buf: &[u8]) -> Result<usize, Error> {
        if block_id > 7 {
            Err(Error::OutOfBounds)
        } else {
            // Check so the block can still be written
            self.blocks[block_id].write_access()?;

            // Enable write
            self.nvmc.0.config.write(|w| w.wen().wen());

            let mut block_ptr = unsafe { self.page_start.add(BLOCK_SIZE_BYTES / 4 * block_id) };

            let size = buf.len().min(BLOCK_SIZE_BYTES);

            // Write buffer
            let mut chunks = buf[..size].chunks_exact(4);

            for chunk in &mut chunks {
                // Create words from each chunk
                let word = u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);

                // Write
                unsafe {
                    ptr::write_volatile(block_ptr, word);
                    block_ptr = block_ptr.add(1);
                }

                // Wait for operation to complete
                while self.nvmc.0.ready.read().ready().is_busy() {}
            }

            let remainder = chunks.remainder();

            if remainder.len() > 0 {
                // Create chunks for the remainder
                let mut buf: [u8; 4] = [remainder[0], 0xff, 0xff, 0xff];

                if remainder.len() > 1 {
                    buf[1] = remainder[1];
                }

                if remainder.len() > 2 {
                    buf[2] = remainder[2];
                }

                let word = u32::from_ne_bytes(buf);
                unsafe {
                    ptr::write_volatile(block_ptr, word);
                }

                // Wait for operation to complete
                while self.nvmc.0.ready.read().ready().is_busy() {}
            }

            // Disable write
            self.nvmc.0.config.write(|w| w.wen().ren());

            Ok(size)
        }
    }
}


/// Helper struct to check bounds and generate addresses for flash pages
#[derive(Debug, Copy, Clone)]
struct FlashPage(u8);

impl FlashPage {
    /// Create a flash page from an flash page index
    fn from_index(page_index: usize) -> Result<Self, Error> {
        if (page_index as usize) < NUMBER_OF_PAGES {
            Ok(Self(page_index as u8))
        } else {
            Err(Error::OutOfBounds)
        }
    }

    /// Get the starting address of the flash page
    fn to_address(&self) -> *mut u32 {
        (self.0 as usize * PAGE_SIZE_BYTES) as _
    }
}

/// Type indicating an erased flash page, this is given from the
/// (Nvmc::erase_page)[Nvmc::erase_page]
pub struct ErasedFlashPage<'a, T> {
    /// Each block within the page,
    blocks: [FlashBlock; 8],
    /// Physical address of the start of the flash page
    page_start: *mut u32,
    /// Access to the NVMC
    nvmc: &'a mut Nvmc<T>,
}

/// Helper struct to check number of write bounds
struct FlashBlock {
    n_writes: u8,
}

impl FlashBlock {
    fn new() -> Self {
        Self { n_writes: 0 }
    }

    /// Returns `Ok(())` if there are more writes possible to the flash block
    fn write_access(&mut self) -> Result<(), Error> {
        if (self.n_writes as usize) < MAX_BLOCK_WRITES {
            self.n_writes += 1;
            Ok(())
        } else {
            Err(Error::MaxBlockWritesReached)
        }
    }
}

/// Implemented by all `NVMC` instances
pub trait Instance: Deref<Target = nvmc::RegisterBlock> {}

impl Instance for NVMC {}
