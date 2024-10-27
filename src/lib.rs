#![no_std]
#![feature(doc_cfg)]
#![feature(naked_functions)]
#![feature(riscv_ext_intrinsics)]
#![feature(asm_const)]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate log;

mod consts;
mod detect;
mod percpu;
mod regs;
mod sbi;
mod trap;
mod vcpu;

pub use self::percpu::RISCVPerCpu;
pub use self::vcpu::RISCVVCpu;
pub use detect::detect_h_extension as has_hardware_support;
