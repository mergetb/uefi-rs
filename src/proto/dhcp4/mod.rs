//! DHCP4 protocol

use crate::proto::Protocol;
use crate::{unsafe_guid, Result, Status};
use core::ffi::c_void;
use core::ptr;
use num_enum::{
    TryFromPrimitive,
    IntoPrimitive,
};

/// The DHCP4 protocol is used to collect configuration information for the EFI
/// IPv4 Protocol driver and provide DHCP server and PXE boot server directory
/// services.
#[repr(C)]
#[unsafe_guid("8a219718-4ef5-4761-91c8-c0f04bda9e56")]
#[derive(Protocol)]
pub struct DHCP4 {
    get_mode_data: extern "efiapi" fn(
        this: &mut DHCP4,
        dhcp4_mode_data: *mut ModeData,
    ) -> Status,

    configure: extern "efiapi" fn(
        this: &mut DHCP4,
        dhcp4_cfg_data: *const ConfigData,
    ) -> Status,

    start: extern "efiapi" fn(
        this: &mut DHCP4,
        event: *mut crate::Event,
    ) -> Status,

    renew_rebind: usize,
    release: usize,
    stop: usize,
    build: usize,
    transmit_receive: usize,
    parse: usize,
}

impl DHCP4 {

    /// Returns the current operating mode and cached data packet for the EFI
    /// DHCPv4 Protocol driver.
    pub fn get_mode_data(
        &mut self, dhcp4_mode_data: *mut ModeData,) -> Result<()> {

        (self.get_mode_data)(self, dhcp4_mode_data).into()

    }

    /// Initializes, changes, or resets the operational settings for the EFI
    /// DHCPv4 Protocol driver.
    pub fn configure(
        &mut self, dhcp4_cfg_data: *const ConfigData) -> Result<()> {

        (self.configure)(self, dhcp4_cfg_data).into()

    }

    /// Starts the DHCP configuration process.
    pub fn start(
        &mut self, event: *mut crate::Event) -> Result<()> {

        (self.start)(self, event).into()

    }

}

/// Configuration data for the DHCP4 protocol implementation
#[repr(C)]
pub struct ConfigData {
    /// Number of times to try sending a packet during the
    /// Dhcp4SendDiscover event and waiting for a response during
    /// the Dhcp4RcvdOffer event. (This value is also the number of
    /// entries in the DiscoverTimeout array.) Set to zero to use the
    /// default try counts and timeout values.
    pub discovery_try_count: u32,
    /// Maximum amount of time (in seconds) to wait for returned
    /// packets in each of the retries. Timeout values of zero will default
    /// to a timeout value of one second. Set to NULL to use default
    /// timeout values.
    pub discovery_try_timeout: *const u32,
    /// Number of times to try sending a packet during the
    /// Dhcp4SendRequest event and waiting for a response during
    /// the Dhcp4RcvdAck event before accepting failure. (This value
    /// is also the number of entries in the RequestTimeout array.)
    /// Set to zero to use the default try counts and timeout values.
    pub request_try_count: u32,
    /// Maximum amount of time (in seconds) to wait for return packets
    /// in each of the retries. Timeout values of zero will default to a
    /// timeout value of one second. Set to NULL to use default timeout
    /// values.
    pub request_timeout: *const u32,
    /// For a DHCPDISCOVER, setting this parameter to the previously
    /// allocated IP address will cause the EFI DHCPv4 Protocol driver
    /// to enter the Dhcp4InitReboot state. Also, set this field to
    /// 0.0.0.0 to enter the Dhcp4Init state.For a DHCPINFORM this
    /// parameter should be set to the client network address which was
    /// assigned to the client during a DHCPDISCOVER.
    pub client_address: IPv4Address,
    /// The callback function to intercept various events that occurred in
    /// the DHCP configuration process. Set to NULL to ignore all those
    /// events. Type EFI_DHCP4_CALLBACK is defined below.
    pub dhcp4_callback: Option<extern "efiapi" fn(
        this: *mut DHCP4,
        context: *mut c_void,
        current_state: State,
        dhcp4_event: Event,
        packet: *const Packet,
        new_packet: *mut *mut Packet,
    ) -> Status>,
}

impl Default for ConfigData {
    fn default() -> ConfigData {
        ConfigData{
            discovery_try_count: 0,
            discovery_try_timeout: ptr::null(),
            request_try_count: 0,
            request_timeout: ptr::null(),
            client_address: [0,0,0,0],
            dhcp4_callback: None,
        }
    }
}

impl core::fmt::Debug for ConfigData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ConfigData")
            .field("discovery_try_count", &self.discovery_try_count)
            .field("discovery_try_timeout", &self.discovery_try_timeout)
            .field("request_try_count", &self.request_try_count)
            .field("request_timeout", &self.request_timeout)
            .field("client_address", &self.client_address)
            .finish()
    }
}

/// An IPv4 address as a sequence of bytes.
pub type IPv4Address = [u8;4];

/// A MAC address as a sequence of bytes
pub type MacAddress = [u8;32];

/* XXX cannot do for type alias apparently and type MacAddress([u8;32]) does not
 * fly either
impl core::fmt::Debug for MacAddress {

    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:02X?}", self))
    }

}
*/

/// The states the DHCP4 EFI protocol may be in
#[derive(Debug, PartialEq)]
pub enum State {
    /// The EFI DHCPv4 Protocol driver is stopped.
    Dhcp4Stopped        = 0x0,
    /// The EFI DHCPv4 Protocol driver is inactive.
    Dhcp4Init           = 0x1,
    /// The EFI DHCPv4 Protocol driver is collecting DHCP offer packets from DHCP servers.
    Dhcp4Selecting      = 0x2,
    /// The EFI DHCPv4 Protocol driver has sent the request to the DHCP server and is waiting for a response
    Dhcp4Requesting     = 0x3,
    /// The DHCP configuration has completed.
    Dhcp4Bound          = 0x4,
    /// The DHCP configuration is being renewed and another request has
    /// been sent out, but it has not received a response from the server yet.
    Dhcp4Renewing       = 0x5,
    /// The DHCP configuration has timed out and the EFI DHCPv4
    /// Protocol driver is trying to extend the lease time.
    Dhcp4Rebinding      = 0x6,
    /// The EFI DHCPv4 Protocol driver was initialized with a previously
    /// allocated or known IP address.
    Dhcp4InitReboot     = 0x7,
    /// The EFI DHCPv4 Protocol driver is seeking to reuse the previously
    /// allocated IP address by sending a request to the DHCP server.
    Dhcp4Rebooting      = 0x8
}

impl Default for State {
    fn default() -> State { State::Dhcp4Stopped }
}

/// Events emitted by the DHCP4 EFI protocol
pub enum Event {
  /// The packet to start the configuration sequence is about to be sent.
  Dhcp4SendDiscover   = 0x01,
  /// A reply packet was just received.
  Dhcp4RcvdOffer      = 0x02,
  /// It is time for Dhcp4Callback to select an offer.
  Dhcp4SelectOffer    = 0x03,
  /// A request packet is about to be sent.
  Dhcp4SendRequest    = 0x04,
  /// A DHCPACK packet was received and will be passed to Dhcp4Callback.
  Dhcp4RcvdAck        = 0x05,
  /// A DHCPNAK packet was received and will be passed to Dhcp4Callback.
  Dhcp4RcvdNak        = 0x06,
  /// A decline packet is about to be sent.
  Dhcp4SendDecline    = 0x07,
  /// The DHCP configuration process has completed. No packet is associated with this event.
  Dhcp4BoundCompleted = 0x08,
  /// It is time to enter the Dhcp4Renewing state and to contact the server
  /// that originally issued the network address. No packet is associated with this event.
  Dhcp4EnterRenewing  = 0x09,
  /// It is time to enter the Dhcp4Rebinding state and to contact any server.
  /// No packet is associated with this event.
  Dhcp4EnterRebinding = 0x0a,
  /// The configured IP address was lost either because the lease has expired,
  /// the user released the configuration, or a DHCPNAK packet was received in
  /// the Dhcp4Renewing or Dhcp4Rebinding state. No packet is associated with this event.
  Dhcp4AddressLost    = 0x0b,
  /// The DHCP process failed because a DHCPNAK packet was received or the user
  /// aborted the DHCP process at a time when the configuration was not available yet.
  /// No packet is associated with this event.
  Dhcp4Fail           = 0x0c
}

/// A DHCP4 packet
#[repr(C)]
pub struct PacketDHCP4 {
    /// DHCP packet header.
    pub header: Header,
    /// DHCP magik cookie in network byte order.
    pub magik: u32,
    /// Start of the DHCP packed option data.
    pub option: *mut u8,
}

impl Default for PacketDHCP4 {

    fn default() -> PacketDHCP4 {
        PacketDHCP4 {
            header: Header::default(),
            magik: 0,
            option: ptr::null_mut(),
        }
    }

}

/// DHCP4 packet payload
#[repr(C)]
#[derive(Default)]
pub struct Packet {
  ///
  /// Size of the EFI_DHCP4_PACKET buffer.
  ///
  pub size: u32,
  ///
  /// Length of the EFI_DHCP4_PACKET from the first byte of the Header field
  /// to the last byte of the Option[] field.
  ///
  pub length: u32,

  /// The DHCP4 packet contents
  pub dhcp4: PacketDHCP4,

}

/// EFI_DHCP4_PACKET defines the format of DHCPv4 packets. See RFC 2131 for more information.
#[repr(C)]
pub struct Header {
  /// Message op code / messge type
  pub op_code: OpCode,
  /// Hardware Address Type
  pub hw_type: HardwareType,
  /// Hardware Address Length
  pub hw_addr_len: u8,
  /// Client sets to zero, optionally used by relay agents when booting via a
  /// relay agent.
  pub hops: u8,
  /// Transaction ID, a random number chosen by the client, used by the client
  /// and server to associate messages and responses between a client and a
  /// server.
  pub xid: u32,
  /// Filled in by client, seconds elapsed since client began address 
  /// acquisition or renewal process.
  pub seconds: u16,
  /// Spacer
  pub reserved: u16,
  /// Client IP address from client.
  pub client_addr: IPv4Address,
  /// Client IP address from server.
  pub your_addr: IPv4Address,
  /// IP address of next server in bootstrap.
  pub server_addr: IPv4Address,
  /// Relay agent IP address.
  pub gateway_addr: IPv4Address,
  /// Client hardware address.
  pub client_hw_addr: [u8;16],
  /// Optional server host name, null terminated string.
  pub server_name: [char;64],
  /// Boot file name, null terminated string; "generic" name or null in 
  /// DHCPDISCOVER, fully qualified directory-path name in DHCPOFFER.
  pub bootfile_name: [char;128],
}

impl Default for Header {

    fn default() -> Header {
        Header {
            op_code: OpCode::default(),
            hw_type: HardwareType::default(),
            hw_addr_len: 0,
            hops: 0,
            xid: 0,
            seconds: 0,
            reserved: 0,
            client_addr: [0;4],
            your_addr: [0;4],
            server_addr: [0;4],
            gateway_addr: [0;4],
            client_hw_addr: [0;16], 
            server_name: ['0';64],
            bootfile_name: ['0';128],
        }
    }

}

/// The op code or message type associated with a DHCP header.
#[repr(u8)]
pub enum OpCode {
    /// Identifies a DHCP packet as a boot request
    BootRequest = 1,
    /// Identifies a DHCP packet as a boot reply
    BootReply = 2,
}

impl Default for OpCode {
    fn default() -> OpCode { OpCode::BootRequest }
}

/// Interface hardware type (RFC 1700 page 163)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum HardwareType {
    /// Standard Ethernet
    Ethernet = 1,
    /// Non-standard Ethernet
    ExperimentalEthernet = 2,
    /// Amateur Radio
    AmateurRadioAX25 = 3,
    /// Token ring
    ProteonProNetTokenRing = 4,
    /// Chaos
    Chaos = 5,
    /// IEEE 802 family of protocols (WiFi and others)
    IEEE802 = 6,
    /// ARCNet
    ARCNET = 7,
    /// Hyperchannel
    Hyperchannel = 8,
    /// Lanstar
    Lanstar = 9,
}

impl Default for HardwareType {
    fn default() -> HardwareType { HardwareType::Ethernet }
}

/// Mode data associated with a DHCP4 protocol instance.
#[repr(C)]
#[derive(Debug)]
pub struct ModeData {
  /// The EFI DHCPv4 Protocol driver operating state.
  pub state: State,
  /// The configuration data of the current EFI DHCPv4 Protocol driver instance.
  pub config_data: ConfigData,
  /// The client IP address that was acquired from the DHCP server. If it is zero,
  /// the DHCP acquisition has not completed yet and the following fields in this structure are undefined.
  pub client_address: IPv4Address,
  /// The local hardware address.
  pub client_mac_address: MacAddress,
  /// The server IP address that is providing the DHCP service to this client.
  pub server_address: IPv4Address,
  /// The router IP address that was acquired from the DHCP server.
  /// May be zero if the server does not offer this address.
  pub router_address: IPv4Address,
  /// The subnet mask of the connected network that was acquired from the DHCP server.
  pub subnet_mask: IPv4Address,
  /// The lease time (in 1-second units) of the configured IP address.
  /// The value 0xFFFFFFFF means that the lease time is infinite.
  /// A default lease of 7 days is used if the DHCP server does not provide a value.
  pub lease_time: u32,
  /// The cached latest DHCPACK or DHCPNAK or BOOTP REPLY packet. May be NULL if no packet is cached.
  pub reply_packet: *const Packet,
}

impl Default for ModeData {

    fn default() -> ModeData {
        ModeData{
            state: State::default(),
            config_data: ConfigData::default(),
            client_address: [0;4],
            client_mac_address: [0;32],
            server_address: [0;4],
            router_address: [0;4],
            subnet_mask: [0;4],
            lease_time: 0,
            reply_packet: ptr::null(),
        }
    }

}
