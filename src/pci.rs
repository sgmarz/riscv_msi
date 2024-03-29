use core::ptr::write_volatile;
use crate::imsic::IMSIC_M;

// ECAM is hard coded in virt.c to 0x3000_0000
const PCI_ECAM_BASE: usize = 0x3000_0000;
// BARs are reserved space in both 0x4000_0000
// and 0x4_0000_0000. Since we're using RV32I, opt
// for the 32-bit address to avoid a dual cycle read/write
// from PCI system.
const PCI_BAR_BASE: usize = 0x4000_0000;

// Bits for the command register in ECAM space
const COMMAND_REG_MEM_SPACE: u16 = 1 << 1;
const COMMAND_REG_BUS_MASTER: u16 = 1 << 2;

pub static mut PCI_INITIALIZED: bool = false;

pub const MAX_PCI_DEVICES: usize = 4;
pub static mut PCI_DEVICES: [Option<PciDevice>; MAX_PCI_DEVICES] = [None; MAX_PCI_DEVICES];

#[derive(Clone, Copy)]
pub enum PciDevice {
    Nvme(usize)
}

fn pci_add_device(dev: PciDevice) {
    unsafe {
        for i in PCI_DEVICES.iter_mut() {
            if i.is_none() {
                *i = Some(dev);
                return;
            }
        }
    }
    println!("Unable to add PCI device.");
}


#[repr(C)]
#[derive(Copy, Clone)]
struct Type0Ecam {
    pub bar: [u32; 6],
    pub cardbus_cis_pointer: u32,
    pub sub_vendor_id: u16,
    pub sub_device_id: u16,
    pub expansion_rom_addr: u32,
    pub capes_pointer: u8,
    pub reserved0: [u8; 3],
    pub reserved1: u32,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_gnt: u8,
    pub max_lat: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Type1Ecam {
    pub bar: [u32; 2],
    pub primary_bus_no: u8,
    pub secondary_bus_no: u8,
    pub subordinate_bus_no: u8,
    pub secondary_latency_timer: u8,
    pub io_base: u8,
    pub io_limit: u8,
    pub secondary_status: u16,
    pub memory_base: u16,
    pub memory_limit: u16,
    pub prefetch_memory_base: u16,
    pub prefetch_memory_limit: u16,
    pub prefetch_base_upper: u32,
    pub prefetch_limit_upper: u32,
    pub io_base_upper: u16,
    pub io_limit_upper: u16,
    pub capes_pointer: u8,
    pub reserved0: [u8; 3],
    pub expansion_rom_addr: u32,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub bridge_control: u16,
}

#[repr(C)]
union TypeXEcam {
    pub type0: Type0Ecam,
    pub type1: Type1Ecam,
}

#[repr(C)]
struct Capability {
    pub id: u8,
    pub next: u8,
}

#[repr(C)]
struct Ecam {
    pub vendor_id: u16,
    pub device_id: u16,
    pub command_reg: u16,
    pub status_reg: u16,
    pub revision_id: u8,
    pub prog_if: u8,
    pub class_subcode: u8,
    pub class_basecode: u8,
    pub cacheline_size: u8,
    pub latency_timer: u8,
    pub header_type: u8,
    pub bist: u8,
    pub typex: TypeXEcam,
}
impl Ecam {
    pub const fn as_mut_ptr(bus: usize, slot: usize) -> *mut Self {
        assert!(bus < 256 && slot < 32);
        (PCI_ECAM_BASE | (bus << 20) | (slot << 15)) as *mut Self
    }

    pub fn as_mut<'a>(bus: usize, slot: usize) -> &'a mut Self {
        unsafe { Self::as_mut_ptr(bus, slot).as_mut().unwrap() }
    }
}

#[repr(C)]
struct MsixCapability {
    pub cap: Capability,
    pub msgcontrol: u16,
    pub table: u32,
    pub pba: u32,
}

#[repr(C)]
struct MsixTable {
    pub addr: u64,
    pub data: u32,
    pub control: u32,
}

#[repr(C)]
#[allow(dead_code)]
struct MsixPba {
    pub pending: u64,
}

fn pci_setup(bus: usize, slot: usize) {
    let ecam = Ecam::as_mut(bus, slot);
    if ecam.vendor_id == 0xffff {
        // Vendor id 0xFFFF means "not connected"
        return;
    }
    match ecam.header_type {
        0 => pci_setup_type0(bus, slot, ecam),
        1 => pci_setup_type1(bus, slot, ecam),
        _ => panic!("Unknown PCI type {}.", ecam.header_type),
    }
}

fn pci_setup_type0(bus: usize, slot: usize, ecam: &mut Ecam) {
    // Type 0 setup (devices)
    let mut baraddr = PCI_BAR_BASE | (bus << 20) | (slot << 16);
    ecam.command_reg = 0;
    let mut i = 0;
    while i < 6 {
        unsafe {
            let barval = ecam.typex.type0.bar[i];
            if barval == 0 || barval & 1 != 0 {
                // If the bar is all 0s, it is unimplemented
                // If the first bit is not 0, then it is I/O space,
                // which we don't support.
                i += 1;
                continue;
            }
            let bartype = barval >> 1 & 3;
            match bartype {
                0b00 => {
                    // 32-bit BAR
                    let barptr = &mut ecam.typex.type0.bar[i] as *mut u32;
                    barptr.write_volatile(0xFFFF_FFFF);
                    let barsize = !(barptr.read_volatile() & !0xF) + 1;
                    // println!("  32-bit BAR {}, size {} bytes set to 0x{:08x}.", i, barsize, baraddr);
                    barptr.write_volatile(baraddr as u32);
                    baraddr += barsize as usize;
                    i += 1;
                }
                0b10 => {
                    // 64-bit BAR
                    let barptr = &mut ecam.typex.type0.bar[i] as *mut u32 as *mut u64;
                    barptr.write_volatile(0xFFFF_FFFF_FFFF_FFFF);
                    let barsize = !(barptr.read_volatile() & !0xF) + 1;
                    // println!("  64-bit BAR {}, size {} bytes set to 0x{:08x}.", i, barsize, baraddr);
                    barptr.write_volatile(baraddr as u64);
                    baraddr += barsize as usize;
                    i += 2;
                }
                _ => panic!("invalid bar type {}", bartype),
            }
        }
    }

    ecam.command_reg = COMMAND_REG_BUS_MASTER | COMMAND_REG_MEM_SPACE;
    enum_caps(ecam);
    if ecam.device_id == 0x0010 {
        pci_add_device(PciDevice::Nvme(get_bar_addr(ecam, 0)));
    }
}

fn pci_setup_type1(bus: usize, slot: usize, ecam: &mut Ecam) {
    // Type 1 setup (bridges)

    // To make things easy, the bridge is encoded with the bus number
    // which is the same as the slot of the bridge.
    let addrst = PCI_BAR_BASE | (slot << 20);
    let addred = addrst + ((1 << 20) - 1);

    ecam.command_reg = COMMAND_REG_MEM_SPACE;
    ecam.typex.type1.memory_base = (addrst >> 16) as u16;
    ecam.typex.type1.memory_limit = (addred >> 16) as u16;
    ecam.typex.type1.prefetch_memory_base = (addrst >> 16) as u16;
    ecam.typex.type1.prefetch_memory_limit = (addred >> 16) as u16;
    ecam.typex.type1.primary_bus_no = bus as u8;
    ecam.typex.type1.secondary_bus_no = slot as u8;
    ecam.typex.type1.subordinate_bus_no = slot as u8;
}

fn enum_caps(ecam: &Ecam) {
    let eptr = ecam as *const Ecam as *const u8;
    if ecam.status_reg >> 4 & 1 != 1 {
        // No capabilities
        return;
    }
    let mut c = unsafe { ecam.typex.type0.capes_pointer };
    while c != 0 {
        unsafe {
            let cap = eptr.add(c as usize) as *mut Capability;
            c = (*cap).next;

            if (*cap).id == 0x11 {
                // MSI-X capability
                setup_msix(ecam, cap);
            }
        }
    }
}

fn setup_msix(ecam: &Ecam, cap: *mut Capability) {
    let msixcapptr = cap as *mut MsixCapability;
    let msixcap = unsafe { msixcapptr.as_ref().unwrap() };
    let table_offset = msixcap.table & !7;
    let table_bir = msixcap.table & 7;
    let pba_offset = msixcap.pba & !7;
    let pba_bir = msixcap.pba & 7;
    // println!("Table offset: 0x{:08x} on {}, PBA offset: 0x{:08x} on {}.", table_offset, table_bir, pba_offset, pba_bir);
    let tabba = get_bar_addr(ecam, table_bir as usize) + table_offset as usize;
    let pbaba = get_bar_addr(ecam, pba_bir as usize) + pba_offset as usize;
    println!("TAB = 0x{:08x}, PBA = 0x{:08x}", tabba, pbaba);

    // Enable MSI-X by setting bit 15 (MSI-X Enable bit)
    unsafe {
        write_volatile(&mut (*msixcapptr).msgcontrol, 1 << 15);
    }

    let tabsize = unsafe { (msixcapptr.read_volatile().msgcontrol & 0x3FF) + 1 };
    println!("Table size = {}", tabsize);

    let msixtab = unsafe { (tabba as *mut MsixTable).as_mut().unwrap() };
    println!("Control reg = 0x{:08x}", msixtab.control);
    msixtab.addr = IMSIC_M as u64;
    msixtab.data = 31;
    msixtab.control = 0;
}

/// Get the bar address straight from the BAR register. We could store the
/// BAR, but we already have space for it, so why waste the 4 or 8 bytes?
fn get_bar_addr(ecam: &Ecam, which: usize) -> usize {
    assert!(which < 6);
    // Strip off the last four bits which do not contribute to the address
    // and are instead used to denote the size of the BAR as well as where
    // the BAR connects 0 = MMIO, 1 = PIO
    unsafe { ecam.typex.type0.bar[which] as usize & !0xf }
}

pub fn pci_init() {
    if unsafe { PCI_INITIALIZED } {
        println!("PCI subsystem already initialized.");
        return;
    }
    for bus in 0..=4 {
        // Typically, there are 8 bits for the bus number, but not
        // all have to be implemented.
        let slot_start = if bus == 0 { 1 } else { 0 };
        for slot in slot_start..32 {
            pci_setup(bus, slot);
        }
    }
    unsafe {
        PCI_INITIALIZED = true;
    }
}
