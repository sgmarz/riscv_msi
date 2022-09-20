# RISC-V MSI Test in Rust

Testing the new MSIs added by the draft Advanced Interrupt Architecture (AIA) specification.

# Blog Posts

First blog post: [30-June-2022](https://blog.stephenmarz.com/2022/06/30/riscv-imsic/)

Second blog post: [26-July-2022](https://blog.stephenmarz.com/2022/07/26/aplic/)

# Quick Emulator (QEMU)

The MSI controller has been added to the `virt` machine to QEMU. This may require you to upgrade your QEMU.

[Quick Emulator on GitHub](https://github.com/qemu)

## Creating the Hard Drive for PCI test

You need to create a file to attach as the "hard drive". This can be done one of several ways:

`fallocate -l8M hdd.dsk`

or

`dd if=/dev/urandom of=hdd.dsk bs=1M count=8`

# Downloading Rust Toolchain

Get rust here: [rustup.rs](http://rustup.rs)

Make sure the riscv32i-unknown-none-elf target is added.

`rustup target add riscv32i-unknown-none-elf`

# Running

The `run.sh` script controls the parameters to QEMU. This is linked to cargo via `.cargo/config`

Run the test by using cargo:

`cargo run`

or 

`cargo run --release`



