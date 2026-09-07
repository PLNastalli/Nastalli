#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

use bootloader_api::{BootInfo, entry_point};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    novaos_kernel::start(boot_info)
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    novaos_kernel::panic(info)
}
