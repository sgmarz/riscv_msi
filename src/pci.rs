
const PCI_ECAM_BASE: usize = 0x3000_0000;
const PCI_BAR_BASE: usize = 0x4000_0000;

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
    pub id: u8,
    pub next: u8,
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
struct MsixPba {
    pub pending: u64,
}


fn pci_enum(bus: usize, slot: usize) {
    let ecam = Ecam::as_mut(bus, slot);
    if ecam.vendor_id == 0xffff {
        // Vendor id 0xFFFF means "not connected"
        return;
    }
    println!("PCI Device {}:{}: Type {}, Vendor: 0x{:04x}, Device: 0x{:04x}", bus, slot, ecam.header_type, ecam.vendor_id, ecam.device_id);
    match ecam.header_type {
        0 => pci_setup_type0(ecam),
        1 => pci_setup_type1(ecam),
        _ => panic!("Unknown PCI type {}.", ecam.header_type)
    }
}

fn pci_setup_type0(ecam: &mut Ecam) {

}

fn pci_setup_type1(ecam: &mut Ecam) {

}

pub fn pci_init() {
    for bus in 0..=4 {
        // Typically, there are 8 bits for the bus number, but not
        // all have to be implemented.
        for slot in 0..32 {
            // The slot number is 5 bits

            // Do not enumerate the root
            if bus != 0 || slot != 0 {
                pci_enum(bus, slot);
            }
        }
    }
}

