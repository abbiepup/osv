#![no_std]
#![no_main]
#![feature(abi_riscv_interrupt)]

use core::panic::PanicInfo;
use riscv::interrupt::Trap;
use riscv::interrupt::supervisor::{Exception, Interrupt};
use riscv::register::stvec::{Stvec, TrapMode};
use riscv_peripheral::aclint::mtimer::Mtimer;
use uefi::Status;

riscv_peripheral::clint_codegen!(pub Clint, base 0x0200_0000, mtime_freq 32_768);

#[uefi::entry]
fn efi_main() -> Status {
    let _mmap = unsafe { uefi::boot::exit_boot_services(None) };

    Status(kernel_main(Args { todo: () }))
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(_args: Args) -> usize {
    let stvec = Stvec::new(interrupt_handler as usize, TrapMode::Direct);
    unsafe { riscv::register::stvec::write(stvec) };
    unsafe { riscv::register::sstatus::set_sie() };
    unsafe { riscv::register::sie::set_stimer() };

    loop {
        riscv::asm::wfi();
    }
}

extern "riscv-interrupt-s" fn interrupt_handler() {
    let trap: Trap<Interrupt, Exception> = unsafe {
        riscv::register::scause::read()
            .cause()
            .try_into()
            .unwrap_unchecked()
    };

    match trap {
        Trap::Interrupt(interrupt) => match interrupt {
            Interrupt::SupervisorSoft => {
                unsafe { riscv::register::sip::clear_ssoft() };
            }
            Interrupt::SupervisorTimer => {
                let clint = Clint::new();
                let now = clint.mtimer().mtime().read();
                clint.mtimer().mtimecmp0().write(now + Clint::MTIME_FREQ as u64);
            }
            _ => unimplemented!(),
        },
        Trap::Exception(exception) => match exception {
            _ => unimplemented!(),
        },
    }
}

#[repr(C)]
pub struct Args {
    todo: (),
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    loop {}
}
