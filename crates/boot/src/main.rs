#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

use bootloader_api::{BootInfo, entry_point};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    nastalli_kernel::start(boot_info)
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    nastalli_kernel::panic(info)
}
