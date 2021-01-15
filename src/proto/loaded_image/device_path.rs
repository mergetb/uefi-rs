//! Device path protocol

use crate::{
    proto::Protocol, 
    unsafe_guid, 
    proto::dhcp4::HardwareType,
    alloc::ALLOCATOR,
};
use core::{
    ptr,
    mem::size_of,
    convert::TryFrom,
};
use num_enum::{
    TryFromPrimitive,
    IntoPrimitive,
};

#[cfg(feature = "exts")]
use alloc_api::{
    format,
    alloc::{
        alloc,
        Layout,
        GlobalAlloc,
    },
};

// Memory management ==========================================================

pub struct DevicePathBox { ptr: ptr::Unique<DevicePath> }

impl DevicePathBox {

    fn new(p: *mut DevicePath) -> DevicePathBox {
        DevicePathBox{ ptr: unsafe { ptr::Unique::new_unchecked(p) } }
    }

    pub fn as_ptr(&self) -> *const DevicePath {
        self.ptr.as_ptr()
    }

}

impl core::ops::Deref for DevicePathBox {

    type Target = DevicePath;

    fn deref(&self) -> &DevicePath {
        unsafe { self.ptr.as_ref() }
    }

}

impl core::ops::DerefMut for DevicePathBox {

    fn deref_mut(&mut self) -> &mut DevicePath {
        unsafe { self.ptr.as_mut() }
    }

}

impl Drop for DevicePathBox {

    // https://doc.rust-lang.org/nomicon/destructors.html
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.ptr.as_ptr());
            ALLOCATOR.dealloc(
                self.ptr.as_mut() as *mut _ as *mut u8, 
                Layout::new::<DevicePath>(),
            );
            //TODO drop the entire path
        }
    }

}

// Protocol definitions =======================================================

/// DevicePath protocol. This can be opened on a `LoadedImage.device()` handle
/// using the `HandleProtocol` boot service.
#[repr(C)]
#[unsafe_guid("09576e91-6d3f-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct DevicePath {
    /// Type of device
    pub device_type: DeviceType,
    /// Sub type of device
    pub sub_type: u8,
    /// Tata related to device path
    ///
    /// The device_type and sub_type determine the
    /// kind of data, and it size.
    pub length: [u8; 2],
}

pub struct DevicePathPayload<T> {
    path: DevicePath,
    payload: T
}

/// Device Path Utilities Protocol. Creates and manipulates device paths and 
/// device nodes.
#[repr(C)]
#[unsafe_guid("0379BE4E-D706-437d-B037-EDB82FB772A4")]
#[derive(Protocol)]
pub struct DevicePathUtilities {
    get_device_path_state: usize,
    duplicate_device_path: usize,
    append_device_node: usize,

    append_device_path: extern "efiapi" fn(
        src1: &DevicePath,
        src2: &DevicePath,
    ) -> *mut DevicePath,


    append_device_path_instance: usize,
    get_next_device_path_instance: usize,
    is_device_path_multi_instance: usize,
    create_device_node: usize,
}


// Type and sub-type enums ====================================================

/// Type identifier for a DevicePath
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    /// Hardware Device Path
    Hardware = 0x01,
    /// ACPI Device Path
    ACPI = 0x02,
    /// Messaging Device Path
    Messaging = 0x03,
    /// Media Device Path
    Media = 0x04,
    /// BIOS Boot Specification Device Path
    BIOSBootSpec = 0x05,
    /// End of Hardware Device Path
    End = 0x7F,
}


/// Sub-type identifier for an "End of Hardware" device path
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum EndPathSubType {
    /// End This Instance of a Device Path and start a new Device Path
    EndInstance = 0x01,
    /// End Entire Device Path
    EndEntire = 0xFF,
}

/// Sub-type identifier for a Hardware device path
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum HardwarePathSubType {
    PCI = 1,
    PCCARD = 2,
    MemoryMapped = 3,
    Vendor = 4,
    Controller = 5,
    BMC = 6,
}

/// Sub-type identifier for an ACPI device path
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum ACPIPathSubType {
    ACPI = 1,
    ExpandedACPI = 2,
    ADR = 3,
}

/// Sub-type identifier for a Messageing device path
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum MessagingPathSubType {
    ATAPI =             1,
    SCSI =              2,
    FibreChannel =      3,
    IEEE1394 =          4,
    USB =               5,
    I20 =               6,
    InfiniBand =        9,
    Vendor =            10,
    MAC =               11,
    IPv4 =              12,
    IPv6 =              13,
    UART =              14,
    USBClass =          15,
    WWID =              16,
    LogicalUnit =       17,
    SATA =              18,
    ISCSI =             19,
    VLAN =              20,
    FiberChannelEx =    21,
    SASEx =             22,
    NVME =              23,
    URI =               24,
    UFS =               25,
    SD =                26,
    Bluetooth =         27,
    Wireless =          28,
    EMMC =              29,
}

/// Sub-type identifier for a Media device path
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum MediaPathSubType {
    HardDrive = 1,
    CDROM = 2,
    Vendor = 3,
    File = 4,
    MediaProtocol = 5,
    PWIGFirmware = 6,
    PWIGFirmwareVolume = 7,
    RelativeOffsetRange = 8,
    RAMDisk = 9,
}

/// Sub-type identifier for a Bios Boot Specification device path
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum BIOSBootSpecPathSubType {
    V1_01 = 1,
}

// Device path implementation =================================================

impl DevicePath {
    /// Create a new device path
    pub fn new<T: Payload>(device_type: DeviceType, sub_type: u8, data: T) -> DevicePathBox {

        let sz = size_of::<DevicePath>() + data.len();
        unsafe {
            let p = ptr::NonNull::new_unchecked(
                alloc(Layout::from_size_align_unchecked(sz, 0))
            );

            /*
            // device type
            ptr::write(p.as_ptr().cast(), device_type);

            // sub_type
            let mut off = size_of::<DeviceType>();
            ptr::write(p.as_ptr().add(off).cast(), sub_type);

            // length
            off += size_of::<u8>();
            ptr::write(p.as_ptr().add(off).cast(), 
                (size_of::<DevicePath>() + size_of::<T>()) as u16);

            // data
            off += size_of::<[u8;2]>();
            ptr::write(p.as_ptr().add(off).cast(), data);
            */
            Self::stamp(p.as_ptr(), device_type, sub_type, data);

            DevicePathBox::new(p.as_ptr() as *mut DevicePath)
        }

    }

    pub fn new1<A: Payload>(x: DevicePathPayload<A>) -> DevicePathBox {

        // allocate enough space for the provide path plus the path ending
        let xl = x.len();
        let sz = x.len() + size_of::<DevicePath>();
        unsafe {
            let p = ptr::NonNull::new_unchecked(
                alloc(Layout::from_size_align_unchecked(sz, 0))
            );

            Self::stamp(p.as_ptr(), x.path.device_type, x.path.sub_type, x.payload);
            Self::stamp(
                p.as_ptr().add(xl), 
                DeviceType::End,
                EndPathSubType::EndEntire as u8,
                (),
            );

            DevicePathBox::new(p.as_ptr() as *mut DevicePath)
        }

    }

    pub fn new2<A: Payload,B: Payload>(x: DevicePathPayload<A>, y: DevicePathPayload<B>) -> DevicePathBox {

        // allocate enough space for the provide path plus the path ending
        let xl = x.len();
        let yl = y.len();
        let sz = xl + yl + size_of::<DevicePath>();
        unsafe {
            let p = ptr::NonNull::new_unchecked(
                alloc(Layout::from_size_align_unchecked(sz, 0))
            );

            Self::stamp(p.as_ptr(), x.path.device_type, x.path.sub_type, x.payload);
            Self::stamp(
                p.as_ptr().add(xl), 
                y.path.device_type, 
                y.path.sub_type, 
                y.payload
            );
            Self::stamp(
                p.as_ptr().add(xl+yl),
                DeviceType::End,
                EndPathSubType::EndEntire as u8,
                (),
            );

            DevicePathBox::new(p.as_ptr() as *mut DevicePath)
        }

    }

    pub fn append(a: &DevicePath, b: &DevicePath) -> DevicePathBox {

        let mut alen = 0usize;
        let mut blen = 0usize;

        a.walk(&mut |x| alen += x.len());
        b.walk(&mut |x| blen += x.len());

        let len = alen + blen - size_of::<DevicePath>();

        unsafe {
            let p = ptr::NonNull::new_unchecked(
                alloc(Layout::from_size_align_unchecked(len, 0))
            );

            let off = alen - size_of::<DevicePath>();
            ptr::copy_nonoverlapping(
                a as *const _ as *const u8,
                p.as_ptr(),
                off,
            );
            ptr::copy_nonoverlapping(
                b as *const _ as *const u8,
                p.as_ptr().add(off),
                blen,
            );

            DevicePathBox::new(p.as_ptr() as *mut DevicePath)
        }


    }

    fn stamp<T: Payload>(p: *mut u8, device_type: DeviceType, sub_type: u8, data: T) {
        unsafe {
            // device type
            ptr::write(p.cast(), device_type);

            // sub_type
            let mut off = size_of::<DeviceType>();
            ptr::write(p.add(off).cast(), sub_type);

            // length
            off += size_of::<u8>();
            ptr::write(p.add(off).cast(), 
                (size_of::<DevicePath>() + data.len()) as u16);

            // data
            off += size_of::<[u8;2]>();
            //ptr::write(p.add(off).cast(), data);
            if data.len() > 0 {
                ptr::copy_nonoverlapping(
                    data.ptr(),
                    p.add(off) as *mut _ as *mut u8,
                    data.len(),
                );
            }

        }
    }

    pub fn payload<P>(&self) -> &P {

        unsafe { 
            let mut p = self as *const _ as *const u8;
            p = p.add(size_of::<DevicePath>());
            &*(p as *const _ as *const P)
        }

    }

    pub fn walk(&self, f: &mut dyn FnMut(&DevicePath)) {

        f(self);
        match self.next() {
            Some(x) => { x.walk(f) }
            None => { }
        }

    }

    pub fn next(&self) -> Option<&DevicePath> {

        if self.device_type == DeviceType::End {

            match EndPathSubType::try_from(self.sub_type) {

                Ok(EndPathSubType::EndInstance) => { 
                    let len = u16::from_le_bytes(self.length) as usize;

                    unsafe {
                        let p = (self as *const Self as *const u8).add(len);
                        Some(&*(p as *const DevicePath))
                    }

                },

                Ok(EndPathSubType::EndEntire) | Err(_) => None

            }

        }
        else {
            let len = u16::from_le_bytes(self.length) as usize;

            unsafe {
                let p = (self as *const Self as *const u8).add(len);
                Some(&*(p as *const DevicePath))
            }
        }


    }

    pub fn len(&self) -> usize { u16::from_le_bytes(self.length) as usize }

}

impl<T: Payload> DevicePathPayload<T> {

    pub fn create(device_type: DeviceType, sub_type: u8, payload: T) -> Self {

        DevicePathPayload{
            path: DevicePath {
                device_type: device_type,
                sub_type: sub_type,
                length: ((size_of::<DevicePath>() + payload.len()) as u16).to_le_bytes(),
            },
            payload: payload,
        }

    }

    pub fn len(&self) -> usize {
        u16::from_le_bytes(self.path.length) as usize
    }


}

// Device path utilities implementation =======================================

impl DevicePathUtilities {

   pub fn append_device_path(
       &self, src1: &DevicePath, src2: &DevicePath) -> *mut DevicePath {

       (self.append_device_path)(src1, src2)

   }

}

// Payloads ===================================================================

// Common traits all payload types must implement, necessitated by the fact
// that payloads have a packed representation but not all payloads are
// necessarily sized at compile time.
pub trait Payload {

    fn len(&self) -> usize;
    fn ptr(&self) -> *const u8;

}

impl Payload for () {

    fn len(&self) -> usize { 0 }
    fn ptr(&self) -> *const u8 { ptr::null() }

}

/// This Device Path contains ACPI Device IDs that represent a device’s Plug
/// and Play Hardware ID and its corresponding unique persistent ID. The ACPI
/// IDs are stored in the ACPI _HID, _CID, and _UID device identification
/// objects that are associated with a device. The ACPI Device Path contains
/// values that must match exactly the ACPI name space that is provided by the
/// platform firmware to the operating system. Refer to the ACPI specification
/// for a complete description of the.  _HID, _CID, and _UID device
/// identification objects.
#[repr(C)]
pub struct ACPIDevicePath {
    /// Device’s PnP hardware ID stored in a numeric 32-bit
    /// compressed EISA-type ID. This value must match the
    /// corresponding _HID in the ACPI name space.
    pub hid: u32,
    /// Unique ID that is required by ACPI if two devices have the
    /// same _HID. This value must also match the corresponding
    /// _UID/_HID pair in the ACPI name space. Only the 32-bit
    /// numeric value type of _UID is supported; thus strings must
    /// not be used for the _UID in the ACPI name space.
    pub uid: u32,
}

impl Payload for ACPIDevicePath {

    fn len(&self) -> usize { size_of::<Self>() }
    fn ptr(&self) -> *const u8 { &self.hid as *const _ as *const u8 }
}

#[repr(C)]
pub struct MACDevicePath {
    pub address: [u8;32],
    pub iftype: HardwareType,
}

impl Payload for MACDevicePath {

    fn len(&self) -> usize { size_of::<Self>() }
    fn ptr(&self) -> *const u8 { &self.address as *const _ as *const u8 }
}

#[repr(C)]
pub struct PCIDevicePath {
    pub function: u8,
    pub device: u8,
}

impl Payload for PCIDevicePath {

    fn len(&self) -> usize { size_of::<Self>() }
    fn ptr(&self) -> *const u8 { &self.function as *const _ as *const u8 }
}

#[repr(C)]
pub struct IPv4DevicePath {
    pub local_ip: [u8;4],
    pub remote_ip: [u8;4],
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: IPProtocol,
    pub static_ip: StaticIPAddr,
    pub gateway_ip: [u8;4],
    pub subnet_mask: [u8;4],
}

impl Payload for IPv4DevicePath {

    fn len(&self) -> usize { size_of::<Self>() }
    fn ptr(&self) -> *const u8 { &self.local_ip as *const _ as *const u8 }

}

#[repr(C)]
pub struct URIDevicePath {
    pub uri: &'static str, //TODO probably non static lifetime better
}

impl Payload for URIDevicePath {

    fn len(&self) -> usize { self.uri.len() }
    fn ptr(&self) -> *const u8 { self.uri as *const _ as *const u8 }

}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum  StaticIPAddr {
    DHCP = 0,
    Static = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[allow(non_camel_case_types)]
pub enum IPProtocol {
    HOPOPT              = 0,
    ICMP                = 1,
    IGMP                = 2,
    GGP                 = 3,
    IPv4                = 4,
    ST                  = 5,
    TCP                 = 6,
    CBT                 = 7,
    EGP                 = 8,
    IGP                 = 9,
    BBN_RCC_MON         = 10,
    NVP_II              = 11,
    PUP                 = 12,
    ARGUS               = 13,
    EMCON               = 14,
    XNET                = 15,
    CHAOS               = 16,
    UDP                 = 17,
    MUX                 = 18,
    DCN_MEAS            = 19,
    HMP                 = 20,
    PRM                 = 21,
    XNS_IDP             = 22,
    TRUNK_1             = 23,
    TRUNK_2             = 24,
    LEAF_1              = 25,
    LEAF_2              = 26,
    RDP                 = 27,
    IRTP                = 28,
    ISO_TP4             = 29,
    NETBLT              = 30,
    MFE_NSP             = 31,
    MERIT_INP           = 32,
    DCCP                = 33,
    _3PC                = 34,
    IDPR                = 35,
    XTP                 = 36,
    DDP                 = 37,
    IDPR_CMTP           = 38,
    TPPP                = 39,
    IL                  = 40,
    IPv6                = 41,
    SDRP                = 42,
    IPv6_Route          = 43,
    IPv6_Frag           = 44,
    IDRP                = 45,
    RSVP                = 46,
    GRE                 = 47,
    DSR                 = 48,
    BNA                 = 49,
    ESP                 = 50,
    AH                  = 51,
    I_NLSP              = 52,
    SWIPE               = 53,
    NARP                = 54,
    MOBILE              = 55,
    TLSP                = 56,
    SKIP                = 57,
    IPv6_ICMP           = 58,
    IPv6_NoNxt          = 59,
    IPv6_Opts           = 60,
    ANYHOST             = 61,
    CFTP                = 62,
    ANYLOCAL            = 63,
    SAT_EXPAK           = 64,
    KRYPTOLAN           = 65,
    RVD                 = 66,
    IPPC                = 67,
    ANYDFS              = 68,
    SAT_MON             = 69,
    VISA                = 70,
    IPCV                = 71,
    CPNX                = 72,
    CPHB                = 73,
    WSN                 = 74,
    PVP                 = 75,
    BR_SAT_MON          = 76,
    SUN_ND              = 77,
    WB_MON              = 78,
    WB_EXPAK            = 79,
    ISO_IP              = 80,
    VMTP                = 81,
    SECURE_VMTP         = 82,
    VINES               = 83,
    TTP_IPTM            = 84,
    NSFNET_IGP          = 85,
    DGP                 = 86,
    TCF                 = 87,
    EIGRP               = 88,
    OSPFIGP             = 89,
    Sprite_RPC          = 90,
    LARP                = 91,
    MTP                 = 92,
    AX25                = 93,
    IPIP                = 94,
    MICP                = 95,
    SCC_SP              = 96,
    ETHERIP             = 97,
    ENCAP               = 98,
    ANYPE               = 99,
    GMTP                = 100,
    IFMP                = 101,
    PNNI                = 102,
    PIM                 = 103,
    ARIS                = 104,
    SCPS                = 105,
    QNX                 = 106,
    A_N                 = 107,
    IPComp              = 108,
    SNP                 = 109,
    Compaq_Peer         = 110,
    IPX_in_IP           = 111,
    VRRP                = 112,
    PGM                 = 113,
    ANYZH               = 114,
    L2TP                = 115,
    DDX                 = 116,
    IATP                = 117,
    STP                 = 118,
    SRP                 = 119,
    UTI                 = 120,
    SMP                 = 121,
    SM                  = 122,
    PTP                 = 123,
    ISIS_over_IPv4      = 124,
    FIRE                = 125,
    CRTP                = 126,
    CRUDP               = 127,
    SSCOPMCE            = 128,
    IPLT                = 129,
    SPS                 = 130,
    PIPE                = 131,
    SCTP                = 132,
    FC                  = 133,
    RSVP_E2E_IGNORE     = 134,
    Mobility_Header     = 135,
    UDPLite             = 136,
    MPLS_in_IP          = 137,
    manet               = 138,
    HIP                 = 139,
    Shim6               = 140,
    WESP                = 141,
    ROHC                = 142,
    Ethernet            = 143,
    //UNASSIGNED        = 144-252,
    EXP0                = 253,
    EXP2                = 254,
    Reserved            = 255,
}

// Debug ======================================================================

impl core::fmt::Debug for DevicePath {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {

        let mut d = f.debug_struct("DevicePathHeader");
        d.field("device_type", &self.device_type);

        match self.device_type {

            DeviceType::Hardware => {
                match HardwarePathSubType::try_from(self.sub_type) {
                    Ok(s) => {
                        d.field("sub_type", &s);
                        match s {
                            HardwarePathSubType::PCI => {
                                let pl = self.payload::<PCIDevicePath>();
                                d.field("data", &pl);
                            }
                            _ => {}
                        }
                    }
                    _ => { d.field("sub_type", &self.sub_type); }
                };
            },

            DeviceType::ACPI => {
                match ACPIPathSubType::try_from(self.sub_type) {
                    Ok(s) => {
                        d.field("sub_type", &s);
                        match s {
                            ACPIPathSubType::ACPI => {
                                let pl = self.payload::<ACPIDevicePath>();
                                d.field("data", &pl);
                            }
                            _ => {}
                        }
                    },
                    _ => { d.field("sub_type", &self.sub_type); },
                };
            },

            DeviceType::Messaging => {
                match MessagingPathSubType::try_from(self.sub_type) {
                    Ok(s) => {
                        d.field("sub_type", &s);
                        match s {
                            MessagingPathSubType::MAC => {
                                let pl = self.payload::<MACDevicePath>();
                                d.field("data", &pl);
                            }
                            MessagingPathSubType::IPv4 => {
                                let pl = self.payload::<IPv4DevicePath>();
                                d.field("data", &pl);
                            }
                            MessagingPathSubType::URI => {
                                /*
                                let pl = self.payload::<URIDevicePath>();
                                d.field("data", &pl);
                                */
                            }
                            _ => {}
                        }
                    },
                    _ => { d.field("sub_type", &self.sub_type); },
                };
            },

            DeviceType::Media => {
                match MediaPathSubType::try_from(self.sub_type) {
                    Ok(s) => d.field("sub_type", &s),
                    _ => d.field("sub_type", &self.sub_type),
                };
            },

            DeviceType::BIOSBootSpec => {
                match BIOSBootSpecPathSubType::try_from(self.sub_type) {
                    Ok(s) => d.field("sub_type", &s),
                    _ => d.field("sub_type", &self.sub_type),
                };
            },

            DeviceType::End => {
                match EndPathSubType::try_from(self.sub_type) {
                    Ok(s) => d.field("sub_type", &s),
                    _ => d.field("sub_type", &self.sub_type),
                };
            },
        };
        

        d.field("length", &u16::from_le_bytes(self.length));

        d.finish()
    }
}

impl core::fmt::Debug for ACPIDevicePath {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {

        // First three 5 bit chunks are ASCII codes starting from the capital
        // letter A yes, that's right, 5 bits, no joke and an implicit offset 
        // of 64, lovely...
        let a = (64 + (self.hid & 0b11111)) as u8 as char;
        let b = (64 + ((self.hid>>5) & 0b11111)) as u8 as char;
        let c = (64 + ((self.hid>>10) & 0b11111)) as u8 as char;

        // shart out the remaning EISA id as a 16 bit hex dump
        let n = (self.hid >> 16) as u16;
        //info!("ACPI ID: {}{}{}{:04X}", a, b, c, n);

        let hid = format!("{}{}{}{:X}", a, b, c, n);
        let uid = format!("{:X}", self.uid);
        f.debug_struct("ACPIDevicePath")
            .field("hid", &hid)
            .field("uid", &uid)
            .finish()

    }

}

impl core::fmt::Debug for MACDevicePath {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {

        let mac = format!("{:x}:{:x}:{:x}:{:x}:{:x}:{:x}", 
            &self.address[0],
            &self.address[1],
            &self.address[2],
            &self.address[3],
            &self.address[4],
            &self.address[5],
        );

        f.debug_struct("MACDevicePath")
            .field("address", &mac)
            .field("iftype", &self.iftype)
            .finish()

    }

}

impl core::fmt::Debug for IPv4DevicePath {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {

        f.debug_struct("IPv4DevicePath")
            .field("local_ip", &self.local_ip)
            .field("remote_ip", &self.remote_ip)
            .field("local_port", &self.local_port)
            .field("remote_port", &self.remote_port)
            .field("protocol", &self.protocol)
            .field("static_ip", &self.static_ip)
            .field("gateway_ip", &self.gateway_ip)
            .field("subnet_mask", &self.subnet_mask)
            .finish()

    }

}

impl core::fmt::Debug for PCIDevicePath {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {

        f.debug_struct("PCIDevicePath")
            .field("function", &self.function)
            .field("device", &self.device)
            .finish()

    }
}

/*
impl core::fmt::Debug for URIDevicePath {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {

        f.debug_struct("URIDevicePath")
            .field("uri", &self.uri)
            .finish()

    }

}
*/
