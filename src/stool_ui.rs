use druid::Data;
use std::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum Protocol {
    Lines,
    Raw,
}

// FIXME should probably not be done like this
#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum DruidDataBits {
    Eight,
    Seven,
    Six,
    Five,
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum DruidFlowControl {
    Hardware,
    Software,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum DruidParity {
    Even,
    Odd,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum DruidStopBits {
    One,
    Two,
}
// END FIXME

#[derive(Debug, Clone, PartialEq, Data)]
pub struct OpenMessage {
    pub port_name: String,
    pub baud_rate: String,
    pub data_bits: DruidDataBits,
    pub flow_control: DruidFlowControl,
    pub parity: DruidParity,
    pub stop_bits: DruidStopBits,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuiMessage {
    Open(OpenMessage),
    Close,
    UpdateProtocol(Protocol),
    Write(Vec<u8>),
    Shutdown,
}
