//! Device path protocol

use crate::{proto::Protocol, unsafe_guid};
use core::ffi::c_void;

/// DevicePath protocol. This can be opened on a `LoadedImage.device()` handle
/// using the `HandleProtocol` boot service.
#[repr(C)]
#[unsafe_guid("09576e91-6d3f-11d2-8e39-00a0c969723b")]
#[derive(Protocol)]
pub struct DevicePath {
    /// Type of device
    pub device_type: DeviceType,
    /// Sub type of device
    pub sub_type: DeviceSubType,
    /// Tata related to device path
    ///
    /// The device_type and sub_type determine the
    /// kind of data, and it size.
    pub length: [u8; 2],
}

/// Type identifier for a DevicePath
#[repr(u8)]
#[derive(Debug)]
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

/// Sub-type identifier for a DevicePath
#[repr(u8)]
#[derive(Debug)]
pub enum DeviceSubType {
    /// End This Instance of a Device Path and start a new Device Path
    EndInstance = 0x01,


    /// Uniform Resource Identifier (URI RFC 3986)
    // the UEFI spec presents these in decimal as oppsed to hex, not sure if we
    // want to maintain that here or just put everything in hex.
    URI = 24, 

    /// End Entire Device Path
    EndEntire = 0xFF,
}

/// Uniform Resource Identifier (URI) Device Path SubType
#[repr(C)]
pub struct URIDevicePath {
    /// Common DevicePath header.
    pub header: DevicePath,
    /// URI string
    pub uri: *const c_void,
}
