// SPDX-License-Identifier: GPL-2.0

//! Driver for Raspi GPIO LED.
//!
//! Bashd on C driver:../gpio-sample-rpbled.c
//!
//! only build-in:
//! make LLVM=-17 O=build_4b ARCH=arm64 
//!
//! How to use in qemu:
//! / # sudo insmod rust_miscdev.ko
//! / # sudo cat /proc/misc  -> c 10 122
//! / # sudo chmod 777 /dev/rust_misc
//! / # sudo echo "hello" > /dev/rust_misc
//! / # sudo cat /dev/rust_misc  -> Hello
//! 

#![no_std]

// TODO
// 参考 drivers/gpio/exercise-3-gpio/gpio-sample-rpbled.c代码
// C代码里面用到的字符设备，还是用练习二的Msicdev来实现就可以了
// OSLayer已经集成，但是Adapter driver和Pure driver层的代码需要自己实现
// 最好通过树莓派开发板来查看相应的GPIO控制效果
//
use led_sample::RpiGpioPort;
use core::result::Result::Ok;
use core::ops::Deref;
use kernel::prelude::*;
use kernel::{
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    sync::{Arc, ArcBorrow},
    sync::Mutex,
    miscdev, 
    pin_init,
    new_mutex,
    fmt,
    bindings,
};

module! {
    type: RustMiscDev,
    name: "rust_led",
    author: "kami",
    description: "Rust exercise 003",
    license: "GPL",
}

const GLOBALMEM_SIZE: usize = 0x1000;
const GPIO_SIZE:u64 = 0xB4;
const BCM2837_GPIO_BASE:u64 = 0x3F200000;

#[pin_data]
struct RustMiscdevData {
    #[pin]
    inner: Mutex<[u8;GLOBALMEM_SIZE]>,
    rpi_gpio_port: RpiGpioPort
}

impl RustMiscdevData {
    fn try_new() -> Result<Arc<Self>>{
        pr_info!("rust miscdevice created\n");
        let mapped_base = unsafe { bindings::ioremap(BCM2837_GPIO_BASE, GPIO_SIZE) };
        Ok(Arc::pin_init(
            pin_init!(Self {
                inner <- new_mutex!([0u8;GLOBALMEM_SIZE]),
                rpi_gpio_port: RpiGpioPort::new(mapped_base as *mut u8)
            })
        )?)
    }
}

unsafe impl Sync for RustMiscdevData {}
unsafe impl Send for RustMiscdevData {}

// unit struct for file operations
struct RustFile;

#[vtable]
impl file::Operations for RustFile {
    type Data = Arc<RustMiscdevData>;
    type OpenData = Arc<RustMiscdevData>;

    fn open(shared: &Arc<RustMiscdevData>, _file: &file::File) -> Result<Self::Data> {
        pr_info!("open in miscdevice\n",);
        //TODO
        //todo!()
        return Ok(shared.clone());
    }

    fn read(
        shared: ArcBorrow<'_, RustMiscdevData>,
        _file: &File,
        writer: &mut impl IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        pr_info!("read in miscdevice\n");
        //TODO
        //todo!()
        let data=shared.deref().inner.lock();
        if offset>=GLOBALMEM_SIZE as u64 {
            return Ok(0); //EOF
        }
        let read_length = core::cmp::min(GLOBALMEM_SIZE as u64-offset, writer.len() as u64);
        writer.write_slice(&data[offset as usize..offset as usize + read_length as usize])?;
        Ok(read_length as usize)
    }

    fn write(
        shared: ArcBorrow<'_, RustMiscdevData>,
        _file: &File,
        reader: &mut impl IoBufferReader,
        offset: u64,
    ) -> Result<usize> {
        //pr_info!("write in miscdevice\n");
        //TODO
        //todo!()
        if offset>=GLOBALMEM_SIZE as u64 {
            return Ok(0);
        }
        let mut data = shared.deref().inner.lock();
        let write_length = core::cmp::min(GLOBALMEM_SIZE as u64-offset, reader.len() as u64);
        reader.read_slice(&mut data[offset as usize..offset as usize + write_length as usize])?;
        let mut rpi_gpio_port=shared.deref().rpi_gpio_port;
        let _ =rpi_gpio_port.direction_output(17); //set to output
        match data[offset as usize] {
            48 => {
                pr_info!("Led Off\n");
                let _ = rpi_gpio_port.set_value(17,0);
            }
            49 => {
                pr_info!("Led On\n");
                let _ = rpi_gpio_port.set_value(17,1);
            }
            _ => {
                return Err(EINVAL);
            }
        }
        
        Ok(write_length as usize)
    }

    fn release(_data: Self::Data, _file: &File) {
        pr_info!("release in miscdevice\n");
    }
}

struct RustMiscDev {
    _dev: Pin<Box<miscdev::Registration<RustFile>>>,
}

impl kernel::Module for RustMiscDev {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust miscdevice device for led init\n");

        let data: Arc<RustMiscdevData> = RustMiscdevData::try_new()?;

        let misc_reg = miscdev::Registration::new_pinned(fmt!("rust_led"), data)?;

        Ok(RustMiscDev { _dev: misc_reg })
    }
}

impl Drop for RustMiscDev {
    fn drop(&mut self) {
        pr_info!("Rust miscdevice device led exit\n");
    }
}