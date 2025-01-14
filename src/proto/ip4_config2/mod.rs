//! IP4 config2 protocol

use crate::proto::{
    Protocol,
    dhcp4::{
        MacAddress,
        IPv4Address
    }
};
use crate::{unsafe_guid, Result, Status, Event};
use core::{
    ptr,
    ffi::c_void,
};
use log::info;

/// The EFI_IP4_CONFIG2_PROTOCOL provides the mechanism to set and get various types of
/// configurations for the EFI IPv4 network stack.
#[repr(C)]
#[unsafe_guid("5b446ed1-e30b-4faa-871a-3654eca36080")]
#[derive(Protocol)]
pub struct IP4Config2 {

    set_data: extern "efiapi" fn(
        this: &mut IP4Config2,
        data_type: DataType,
        data_size: usize,
        data: *const c_void,
    ) -> Status,

    get_data: extern "efiapi" fn(
        this: &mut IP4Config2,
        data_type: DataType,
        data_size: usize,
        data: *mut c_void,
    ) -> Status,

    register_data_notify: extern "efiapi" fn(
        this: &mut IP4Config2,
        data_type: DataType,
        event: Event,
    ) -> Status,
        

    unregister_data_notify: usize,
}

impl IP4Config2 {
    /// Set the configuration for the EFI IPv4 network stack running on
    /// the communication device this EFI IPv4 Configuration II
    /// Protocol instance manages. See the SetData() function
    /// description.
    pub fn set_data(
        &mut self, data_type: DataType, size: usize, data: *const c_void
    ) -> Result<()> {

        (self.set_data)(self, data_type, size, data).into()

    }

    pub fn get_data(
        &mut self, data_type: DataType, size: usize, data: *mut c_void
    ) -> Result<()> {

        (self.get_data)(self, data_type, size, data).into()

    }

    /// Register an event that is to be signaled whenever a configuration
    /// process on the specified configuration data is done.
    pub fn register_data_notify(
        &mut self, data_type: DataType, event: Event) -> Result<()> {

        (self.register_data_notify)(self, data_type, event).into()

    }
}

/// A unique key that defines what sort of data to get or set in an IPv4 config.
#[repr(C)]
pub enum DataType {
    /// The interface information of the communication device this EFI
    /// IPv4 Configuration II Protocol instance manages. This type of
    /// data is read only. The corresponding Data is of type
    /// EFI_IP4_CONFIG2_INTERFACE_INFO.
    InterfaceInfo,
    /// The general configuration policy for the EFI IPv4 network stack
    /// running on the communication device this EFI IPv4
    /// Configuration II Protocol instance manages. The policy will
    /// affect other configuration settings. The corresponding Data is of
    /// type EFI_IP4_CONFIG2_POLICY.
    Policy,
    /// The station addresses set manually for the EFI IPv4 network
    /// stack. It is only configurable when the policy is
    /// Ip4Config2PolicyStatic. The corresponding Data is of
    /// type EFI_IP4_CONFIG2_MANUAL_ADDRESS.
    ManualAddress,
    /// The gateway addresses set manually for the EFI IPv4 network
    /// stack running on the communication device this EFI IPv4
    /// Configuration II Protocol manages. It is not configurable when
    /// the policy is Ip4Config2PolicyDhcp. The gateway
    /// addresses must be unicast IPv4 addresses. The corresponding
    /// Data is a pointer to an array of EFI_IPv4_ADDRESS
    /// instances.
    Gateway,
    /// The DNS server list for the EFI IPv4 network stack running on
    /// the communication device this EFI IPv4 Configuration II
    /// Protocol manages. It is not configurable when the policy is
    /// Ip4Config2PolicyDhcp.The DNS server addresses must be
    /// unicast IPv4 addresses. The corresponding Data is a pointer to
    /// an array of EFI_IPv4_ADDRESS instances.
    DnsServer,
    /// Identifies an uppper bound on data types
    Maximum
}

/// EFI_IP4_CONFIG2_POLICY
#[repr(C)]
pub enum Policy {
  /// Under this policy, the Ip4Config2DataTypeManualAddress,
  /// Ip4Config2DataTypeGateway and Ip4Config2DataTypeDnsServer configuration
  /// data are required to be set manually. The EFI IPv4 Protocol will get all
  /// required configuration such as IPv4 address, subnet mask and
  /// gateway settings from the EFI IPv4 Configuration II protocol.
  Static,
  /// Under this policy, the Ip4Config2DataTypeManualAddress,
  /// Ip4Config2DataTypeGateway and Ip4Config2DataTypeDnsServer configuration data are
  /// not allowed to set via SetData(). All of these configurations are retrieved from DHCP
  /// server or other auto-configuration mechanism.
  Dhcp,
  /// Identifies and upper bound on data types
  Maximum
}

#[repr(C)]
#[derive(Debug)]
pub struct InterfaceInfo {
    pub name: [u16; 32],
    pub iftype: u8,
    pub hw_address_size: u32,
    pub hw_address: MacAddress,
    pub station_address: IPv4Address,
    pub subnet_mask: IPv4Address,
    pub route_table_size: u32,
    pub route_table: *mut RouteTableEntry,
}

impl InterfaceInfo {

    pub fn new() -> InterfaceInfo {
        InterfaceInfo{
            name: [0;32],
            iftype: 0,
            hw_address_size: 0,
            hw_address: [0;32],
            station_address: [0;4],
            subnet_mask: [0;4],
            route_table_size: 0,
            route_table: ptr::null_mut(),
        }
    }

    pub fn dump_route_table(&self) {

        for i in 0..(self.route_table_size as usize) {

            unsafe { 
                info!("{:?}", &*self.route_table.add(i));
            }

        }

    }
}

#[repr(C)]
#[derive(Debug)]
pub struct RouteTableEntry {
    pub subnet_address: IPv4Address,
    pub subnet_mask: IPv4Address,
    pub gateway_address: IPv4Address,
}

