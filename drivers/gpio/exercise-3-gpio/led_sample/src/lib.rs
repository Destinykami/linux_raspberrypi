#![no_std]
#![feature(const_ptr_as_ref)]
#![feature(const_option)]
#![feature(const_nonnull_new)]

use core::ptr::NonNull;
pub(crate) use osl::error::Result;
const __LOG_PREFIX: &[u8] = b"gpio\0";
use osl::log_info;
use kernel::prelude::*;
use tock_registers::{
    interfaces::{Readable, Writeable},
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

#[repr(C)]
#[allow(non_snake_case)]
pub(crate) struct RPiGpioRegisters {
    pub(crate) GPFSEL0: ReadWrite<u32>, // 0x00
    pub(crate) GPFSEL1: ReadWrite<u32>, // 0x04
    pub(crate) GPFSEL2: ReadWrite<u32>, // 0x08
    pub(crate) GPFSEL3: ReadWrite<u32>, // 0x0C
    pub(crate) GPFSEL4: ReadWrite<u32>, // 0x10
    pub(crate) GPFSEL5: ReadWrite<u32>, // 0x14
    reserved: u32,                      // 0x18
    pub(crate) GPSET0: WriteOnly<u32>,  // 0x1c
    pub(crate) GPSET1: WriteOnly<u32>,  // 0x20
    reserved1: u32,
    pub(crate) GPCLR0: WriteOnly<u32>, // 0x28
    pub(crate) GPCLR1: WriteOnly<u32>, // 0x2C
    reserved2: u32,
    pub(crate) GPLEV0: ReadOnly<u32>, // 0x34
    pub(crate) GPLEV1: ReadOnly<u32>, // 0x38
    reserved3: u32,
    pub(crate) GPEDS0: ReadWrite<u32>, // 0x40
    pub(crate) GPEDS1: ReadWrite<u32>, // 0x44
    reserved4: u32,
    pub(crate) GPREN0: ReadWrite<u32>, // 0x4C
    pub(crate) GPREN1: ReadWrite<u32>, // 0x50
    reserved5: u32,
    pub(crate) GPFEN0: ReadWrite<u32>, // 0x58
    pub(crate) GPFEN1: ReadWrite<u32>, // 0x58
    reserved6: u32,
    pub(crate) GPHEN0: ReadWrite<u32>, // 0x64
    pub(crate) GPHEN1: ReadWrite<u32>, // 0x68
    reserved7: u32,
    pub(crate) GPLEN0: ReadWrite<u32>, // 0x70
    pub(crate) GPLEN1: ReadWrite<u32>, // 0x74
    reserved8: u32,
    pub(crate) GPAREN0: ReadWrite<u32>, // 0x7C
    pub(crate) GPAREN1: ReadWrite<u32>, // 0x80
    reserved9: u32,
    pub(crate) GPAFEN0: ReadWrite<u32>, // 0x88
    pub(crate) GPAFEN1: ReadWrite<u32>, // 0x8C
    reserved10: u32,
    pub(crate) GPPUD: ReadWrite<u32>,     // 0x94
    pub(crate) GPPUDCLK0: ReadWrite<u32>, // 0x98
    pub(crate) GPPUDCLK1: ReadWrite<u32>, // 0x9C
    reserved11: u32,
    test: char,
}

#[derive(Copy, Clone)]
pub struct RpiGpioPort {
    regs: NonNull<RPiGpioRegisters>,
}

unsafe impl Send for RpiGpioPort {}
unsafe impl Sync for RpiGpioPort {}

impl RpiGpioPort {
    pub const fn new(base_addr: *mut u8) -> Self {
        Self {
            regs: NonNull::new(base_addr).unwrap().cast(),
        }
    }

    const fn regs(&self) -> &RPiGpioRegisters {
        unsafe { self.regs.as_ref() }
    }

    pub fn direction_output(&mut self, offset: u32) -> Result<()> {
        let fsel_index = offset as usize / 10;  // Calculate the index of the GPFSEL register
        let fsel_shift = (offset % 10) * 3;    // Calculate the bit shift for the specific pin
        let fsel_register = match fsel_index {
            0 => &self.regs().GPFSEL0,
            1 => &self.regs().GPFSEL1,
            2 => &self.regs().GPFSEL2,
            3 => &self.regs().GPFSEL3,
            4 => &self.regs().GPFSEL4,
            5 => &self.regs().GPFSEL5,
            _ => return Err(EFAULT),  // Handle invalid index case
        };
        let mut fsel_value = fsel_register.get();  // Read the current value of the register
        
        // Clear the current function select bits for the pin
        fsel_value &= !(0b111 << fsel_shift);
        // Set the pin function to 001 (output)
        fsel_value |= 0b001 << fsel_shift;
        
        // Write the updated value back to the GPFSEL register
        fsel_register.set(fsel_value);
        
        Ok(())
    }

    /// set gpio port value
    pub fn set_value(&mut self, offset: u32, value: u32) -> Result<()> {
        let set_register = if value == 1 {
            match offset as usize / 32 {
                0 => &self.regs().GPSET0,
                1 => &self.regs().GPSET1,
                _ => return Err(EFAULT),  // Handle invalid index case
            }
        } else {
            match offset as usize / 32 {
                0 => &self.regs().GPCLR0,
                1 => &self.regs().GPCLR1,
                _ => return Err(EFAULT),  // Handle invalid index case
            }
        };
        
        // Set or clear the pin
        set_register.set(1 << (offset % 32));
        
        Ok(())
    }
}
