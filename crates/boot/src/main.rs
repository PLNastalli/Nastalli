#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

use bootloader_api::config::Mapping;
use bootloader_api::{BootInfo, BootloaderConfig, entry_point};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    nastalli_kernel::start(boot_info)
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    nastalli_kernel::panic(info)
}
