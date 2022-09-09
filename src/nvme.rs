use crate::pci::PCI_INITIALIZED;

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

    unsafe {
        NVME_INITIALIZED = true;
    }
}