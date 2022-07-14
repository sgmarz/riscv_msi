//! Advanced Platform Level Interrupt Controller (APLIC)
//! Stephen Marz
//! 1 Jun 2022

// These MMIO values are hard coded in the QEMU virt
// machine.
// M-mode APLIC
const APLIC_M: usize = 0xc00_0000;
// S-mode APLIC
const APLIC_S: usize = 0xd00_0000;
// S-mode interrupt delivery controller
const APLIC_S_IDC: usize = 0xd00_4000;

#[repr(u32)]
#[allow(dead_code)]
enum SourceModes {
    Inactive = 0,
    Detached = 1,
    RisingEdge = 4,
    FallingEdge = 5,
    LevelHigh = 6,
    LevelLow = 7,
}

#[repr(C)]
struct Aplic {
    pub domaincfg: u32,
    pub sourcecfg: [u32; 1023],
    _reserved1: [u8; 0xBC0],

    pub mmsiaddrcfg: u32,
    pub mmsiaddrcfgh: u32,
    pub smsiaddrcfg: u32,
    pub smsiaddrcfgh: u32,
    _reserved2: [u8; 0x30],

    pub setip: [u32; 32],
    _reserved3: [u8; 92],

    pub setipnum: u32,
    _reserved4: [u8; 0x20],

    pub in_clrip: [u32; 32],
    _reserved5: [u8; 92],

    pub clripnum: u32,
    _reserved6: [u8; 32],

    pub setie: [u32; 32],
    _reserved7: [u8; 92],

    pub setienum: u32,
    _reserved8: [u8; 32],

    pub clrie: [u32; 32],
    _reserved9: [u8; 92],

    pub clrienum: u32,
    _reserved10: [u8; 32],

    pub setipnum_le: u32,
    pub setipnum_be: u32,
    _reserved11: [u8; 4088],

    pub genmsi: u32,
    pub target: [u32; 1023],
}

type AplicMode = crate::imsic::PrivMode;

#[allow(dead_code)]
impl Aplic {
    const fn ptr(mode: AplicMode) -> *mut Self {
        let k = match mode {
            AplicMode::Machine => APLIC_M,
            AplicMode::Supervisor => APLIC_S,
        };
        k as *mut Self
    }

    pub fn as_ref<'a>(mode: AplicMode) -> &'a Self {
        unsafe { Self::ptr(mode).as_ref().unwrap() }
    }

    pub fn as_mut<'a>(mode: AplicMode) -> &'a mut Self {
        unsafe { Self::ptr(mode).as_mut().unwrap() }
    }

    /// # Overview
    /// Set the MSI target physical address. This only accepts the lower
    /// 32-bits of an address.
    /// ## Arguments
    /// * `mode` the MSI mode (machine or supervisor)
    /// * `addr` the physical address for messages. This MUST be page aligned.
    pub fn set_msiaddr(&mut self, mode: AplicMode, addr: usize) {
        match mode {
            AplicMode::Machine => {
                self.mmsiaddrcfg = (addr >> 12) as u32;
                self.mmsiaddrcfgh = 0;
            }
            AplicMode::Supervisor => {
                self.smsiaddrcfg = (addr >> 12) as u32;
                self.smsiaddrcfgh = 0;
            }
        }
    }

    /// # Overview
    /// Set the target interrupt to a given hart, guest, and identifier
    /// ## Arguments
    /// * `irq` - the interrupt to set
    /// * `hart` - the hart that will receive interrupts from this irq
    /// * `guest` - the guest identifier to send these interrupts
    /// * `eiid` - the identification number of the irq (usually the same as the irq itself)
    pub fn set_target(&mut self, irq: u32, hart: u32, guest: u32, eiid: u32) {
        assert!(irq > 1 && irq < 1024);
        self.target[irq as usize - 1] = (hart << 18) | (guest << 12) | eiid;
    }

    /// # Overview
    /// Setup a source configuration to a particular mode.
    /// This does NOT delegate the source to a child.
    /// ## Arguments
    /// * `irq` the interrupt number to set
    /// * `mode` the source mode--how the interrupt is triggered.
    pub fn set_sourcecfg(&mut self, irq: u32, mode: SourceModes) {
        assert!(irq > 1 && irq < 1024);
        self.sourcecfg[irq as usize - 1] = mode as u32;
    }

    /// # Overview
    /// Setup a source configuration to delegate an IRQ to a child.
    /// ## Arguments
    /// * `irq` the interrupt number to delegate
    /// * `child` the child to delegate this interrupt to
    pub fn sourcecfg_delegate(&mut self, irq: u32, child: u32) {
        assert!(irq > 1 && irq < 1024);
        self.sourcecfg[irq as usize - 1] = 1 << 10 | child;
    }

    /// # Overview
    /// Set the `domaincfg` register.
    /// ## Arguments
    /// * `bigendian` `true`: the APLIC uses big endian byte order, `false`: the APLIC uses little endian byte order.
    /// * `msimode` `true`: the APLIC will send MSIs for interrupts, `false`: the APLIC will only trigger actual wires.
    /// * `enabled` `true`: this APLIC is enabled and can receive/send interrupts, `false`: the APLIC domain is disabled.
    pub fn set_domaincfg(&mut self, bigendian: bool, msimode: bool, enabled: bool) {
        let enabled = enabled as u32;
        let msimode = msimode as u32;
        let bigendian = bigendian as u32;
        self.domaincfg = (enabled << 8) | (msimode << 2) | bigendian;
    }

    /// # Overview
    /// Set the irq enabled bit to given state
    /// ## Arguments
    /// * `irq` the interrupt number
    /// * `enabled` true: enable interrupt, false: disable interrupt
    pub fn set_ie(&mut self, irq: u32, enabled: bool) {
        if enabled {
            self.setienum = irq;
        } else {
            self.clrienum = irq;
        }
    }

    /// # Overview
    /// Set the irq pending bit to the given state
    /// ## Arguments
    /// * `irq` the interrupt number
    /// * `pending` true: set the bit to 1, false: clear the bit to 0
    pub fn set_ip(&mut self, irq: u32, pending: bool) {
        if pending {
            self.setipnum = irq;
        } else {
            self.clripnum = irq;
        }
    }
}

#[repr(C)]
struct InterruptDeliveryControl {
    pub idelivery: u32,
    pub iforce: u32,
    pub ithreshold: u32,
    pub topi: u32,
    pub claimi: u32,
}

#[allow(dead_code)]
impl InterruptDeliveryControl {
    /// # Overview
    /// Get the IDC portion of a hart's APLIC
    /// # Arguments
    /// `hart` - the HART number for the IDC to get
    /// # Returns
    /// A mutable MMIO pointer to the IDC registers
    const fn ptr(hart: usize) -> *mut Self {
        assert!(hart < 1024);
        (APLIC_S_IDC + hart * 0x20) as *mut Self
    }

    /// # Overview
    /// Get an immutable reference to the IDC registers
    /// # Arguments
    /// `hart` - the HART number for the IDC to get
    /// # Returns
    /// An immutable reference to the IDC area
    pub fn as_ref<'a>(hart: usize) -> &'a Self {
        unsafe { Self::ptr(hart).as_ref().unwrap() }
    }

    /// # Overview
    /// Get a mutable reference to the IDC registers
    /// # Arguments
    /// `hart` - the HART number for the IDC to get
    /// # Returns
    /// A mutable reference to the IDC area
    pub fn as_mut<'a>(hart: usize) -> &'a mut Self {
        unsafe { Self::ptr(hart).as_mut().unwrap() }
    }
}

/// # Overview
/// Intiailize the APLIC system and run a test, including
/// setting up the APLIC to send messages to the IMSIC in
/// supervisor mode.
pub fn aplic_init() {
    // The root APLIC
    let mplic = Aplic::as_mut(AplicMode::Machine);
    // The delgated child APLIC
    let splic = Aplic::as_mut(AplicMode::Supervisor);

    // Enable both the machine and supervisor PLICS
    mplic.set_domaincfg(false, true, true);
    splic.set_domaincfg(false, true, true);

    // Write messages to IMSIC_S
    mplic.set_msiaddr(AplicMode::Supervisor, crate::imsic::IMSIC_S);

    // Delegate interrupt 10 to child 0, which is APLIC_S
    // Interrupt 10 is the UART. So, whenever the UART receives something
    // into its receiver buffer register, it triggers an IRQ #10 to the APLIC.
    mplic.sourcecfg_delegate(10, 0);

    // The EIID is the value that is written to the MSI address
    // When we read TOPEI in IMSIC, it will give us the EIID if it
    // has been enabled.
    splic.set_target(10, 0, 0, 10);

    // Level high means to trigger the message delivery when the IRQ is
    // asserted (high).
    splic.set_sourcecfg(10, SourceModes::LevelHigh);

    // The order is important. QEMU will not allow enabling of the IRQ
    // unless the source configuration is set properly.
    // mplic.set_irq(10, true);
    splic.set_ie(10, true);
}
