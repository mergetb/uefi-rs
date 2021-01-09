//! LoadFile system support protocols

use crate::proto::Protocol;
use crate::{unsafe_guid, Result, Status};
use core::ffi::c_void;
//use std::os::raw::c_char;

/// Allows loading of files from a number of provider drivers.
#[repr(C)]
#[unsafe_guid("4006c0c1-fcb3-403e-996d-4a6c8724e06d")]
#[derive(Protocol)]
pub struct LoadFile<T> {
    load_file:
        extern "efiapi" fn(
            this: &mut LoadFile<T>,
            root: &mut T,
            boot_policy: bool,
            buffer_size: &mut u64,
            buffer: *mut c_void,
        ) -> Status,
}

impl<T> LoadFile<T> {

    /// Load a file from a media device.
    pub fn load_file(
        &mut self, root: &mut T, p: bool, bs: &mut u64, buf: *mut c_void) -> Result<()> {

        (self.load_file)(self, root, p, bs, buf)
            .into_with_val(|| () )

    }

}

/// The devices specific path of a file. Can take a number of forms as
/// described in section 9.3.5 (v2.6) of the UEFI spec.
#[repr(C)]
pub struct DevicePath {

    /// The type of the device path
    /// 0x01 Hardware Device Path.
    /// 0x02 ACPI Device Path.
    /// 0x03 Messaging Device Path.
    /// 0x04 Media Device Path.
    /// 0x05 BIOS Boot Specification Device Path
    /// 0x7F End of Hardware Device Path.
    pub typ: u8,

    /// The subtype that defines the form of data carried by this device path
    /// UEFI spec value from section 9.3.5
    /// 0xFF End Entire Device Path, or
    /// 0x01 End This Instance of a Device Path and start a new Device Path.
    pub sub_typ: u8,

    /// Lenth of the data carried by this device path
    /// Size of typ + sub_typ + length + payload (4 + N)
    pub length: u16,
}

/// Uniform Resource Identifier (URI) Device Path SubType
#[repr(C)]
pub struct URIDevicePath {

    /// Common DevicePath header.
    pub header: DevicePath,

    /// URI string
    pub uri: *const c_void,

}
