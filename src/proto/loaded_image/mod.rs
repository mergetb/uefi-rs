//! Loaded image protocol.
//!
//! This module also contains the corollary type DevicePath, which is
//! used to emulate `EFI_DEVICE_PATH_PROTOCOL`.

mod device_path;
/*
pub use self::device_path::{
    DevicePath, URIDevicePath, DeviceType, EndPathSubType, 
    DevicePath2, ACPIDevicePath, DevicePath2T, DevicePathHeader,
};
*/

pub use self::device_path::*;

use crate::{
    data_types::{CStr16, Char16},
    proto::Protocol,
    table::boot::MemoryType,
    unsafe_guid, Handle, Status,
};
use core::{ffi::c_void, str};

#[cfg(feature = "exts")]
use alloc_api::boxed::Box;

/// The Loaded Image protocol. This can be opened on any image handle using the `HandleProtocol` boot service.
#[repr(C)]
#[unsafe_guid("5b1b31a1-9562-11d2-8e3f-00a0c969723b")]
#[derive(Protocol)]
pub struct LoadedImage {
    revision: u32,
    parent_handle: Handle,
    system_table: *const c_void,

    // Source location of the image
    device_handle: Handle,
    _file_path: *const c_void, // TODO: not supported yet
    _reserved: *const c_void,

    // Image load options
    load_options_size: u32,
    load_options: *const Char16,

    // Location where image was loaded
    image_base: usize,
    image_size: u64,
    image_code_type: MemoryType,
    image_data_type: MemoryType,
    /// This is a callback that a loaded image can use to do cleanup. It is called by the
    /// UnloadImage boot service.
    unload: extern "efiapi" fn(image_handle: Handle) -> Status,
}

/// Errors that can be raised during parsing of the load options.
#[derive(Debug)]
pub enum LoadOptionsError {
    /// The passed buffer is not large enough to contain the load options.
    BufferTooSmall,
    /// The load options are not valid UTF-8.
    NotValidUtf8,
}

impl LoadedImage {
    /// Returns a handle to the storage device on which the image is located.
    pub fn device(&self) -> Handle {
        self.device_handle
    }

    /// Get the load options of the given image. If the image was executed from the EFI shell, or from a boot
    /// option, this is the command line that was used to execute it as a string. If no options were given, this
    /// returns `Ok("")`.
    pub fn load_options<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a str, LoadOptionsError> {
        if self.load_options.is_null() {
            Ok("")
        } else {
            let ucs2_slice = unsafe { CStr16::from_ptr(self.load_options).to_u16_slice() };
            let length =
                ucs2::decode(ucs2_slice, buffer).map_err(|_| LoadOptionsError::BufferTooSmall)?;
            str::from_utf8(&buffer[0..length]).map_err(|_| LoadOptionsError::NotValidUtf8)
        }
    }

    /// Set the load options of the given image.
    #[cfg(feature = "exts")]
    pub fn set_load_options(&mut self, s: &str) -> Result<(), LoadOptionsError> {

        let mut buf = unsafe{ Box::<[u16]>::new_uninit_slice(s.len()).assume_init() };

        let len = ucs2::encode(s, &mut *buf).map_err(|_| LoadOptionsError::BufferTooSmall)?;

        let opts = Box::into_raw(buf) as *const Char16;
        self.load_options = opts;
        self.load_options_size = len as u32;

        Ok(())

    }

    /// Returns the base address and the size in bytes of the loaded image.
    pub fn info(&self) -> (usize, u64) {
        (self.image_base, self.image_size)
    }
}
