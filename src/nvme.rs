use crate::pci::{PCI_INITIALIZED, PCI_DEVICES, PciDevice};

static mut NVME_INITIALIZED: bool = false;

pub fn init() {
    if unsafe { NVME_INITIALIZED } {
        println!("NVMe already initialized.");
        return;
    }
    if unsafe { !PCI_INITIALIZED } {
        println!("PCI has not yet been initialized.");
        return;
    }
    for i in unsafe { PCI_DEVICES.iter() } {
        if let Some(x) = *i {
            match x {
                PciDevice::Nvme(base) => {
                    nvme_setup(base);
                }
            }
        }
    }

    unsafe {
        NVME_INITIALIZED = true;
    }
}

fn nvme_setup(base: usize) {
    println!("NVME @ 0x{:08x}", base);
}
