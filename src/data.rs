use crate::GuiMessage;
use druid::text::RichText;
use druid::{Data, Lens};
use futures::channel::mpsc::UnboundedSender;
use std::collections::VecDeque;
use std::fmt;
use std::ops::Range;
use std::sync::Arc;
use tokio_serial::{DataBits, FlowControl, Parity, StopBits};

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum Protocol {
    Text,
    Raw,
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum DruidDataBits {
    Eight,
    Seven,
    Six,
    Five,
}

impl From<DruidDataBits> for DataBits {
    fn from(data_bits: DruidDataBits) -> Self {
        match data_bits {
            DruidDataBits::Eight => DataBits::Eight,
            DruidDataBits::Seven => DataBits::Seven,
            DruidDataBits::Six => DataBits::Six,
            DruidDataBits::Five => DataBits::Five,
        }
    }
}

impl fmt::Display for DruidDataBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DruidDataBits::Eight => write!(f, "8"),
            DruidDataBits::Seven => write!(f, "7"),
            DruidDataBits::Six => write!(f, "6"),
            DruidDataBits::Five => write!(f, "5"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum DruidFlowControl {
    Hardware,
    Software,
    None,
}

impl From<DruidFlowControl> for FlowControl {
    fn from(flow_control: DruidFlowControl) -> Self {
        match flow_control {
            DruidFlowControl::Hardware => FlowControl::Hardware,
            DruidFlowControl::Software => FlowControl::Software,
            DruidFlowControl::None => FlowControl::None,
        }
    }
}

impl fmt::Display for DruidFlowControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DruidFlowControl::Hardware => write!(f, "Hardware"),
            DruidFlowControl::Software => write!(f, "Software"),
            DruidFlowControl::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum DruidParity {
    Even,
    Odd,
    None,
}

impl From<DruidParity> for Parity {
    fn from(parity: DruidParity) -> Self {
        match parity {
            DruidParity::Even => Parity::Even,
            DruidParity::Odd => Parity::Odd,
            DruidParity::None => Parity::None,
        }
    }
}

impl fmt::Display for DruidParity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DruidParity::Even => write!(f, "Even"),
            DruidParity::Odd => write!(f, "Odd"),
            DruidParity::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum DruidStopBits {
    One,
    Two,
}

impl From<DruidStopBits> for StopBits {
    fn from(stop_bits: DruidStopBits) -> Self {
        match stop_bits {
            DruidStopBits::One => StopBits::One,
            DruidStopBits::Two => StopBits::Two,
        }
    }
}

impl fmt::Display for DruidStopBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DruidStopBits::One => write!(f, "One"),
            DruidStopBits::Two => write!(f, "Two"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Data)]
pub struct OpenMessage {
    pub port_name: String,
    pub baud_rate: u32,
    pub data_bits: DruidDataBits,
    pub flow_control: DruidFlowControl,
    pub parity: DruidParity,
    pub stop_bits: DruidStopBits,
    pub protocol: Protocol,
}
#[derive(Debug, Clone, PartialEq, Data)]
pub enum OutputTag {
    TextIn,
    TextOut,
    RawIn,
    RawOut,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct AppData {
    pub output: RichText,
    pub output_attr: Arc<VecDeque<(Range<usize>, OutputTag)>>,
    pub port_name: Arc<String>,
    pub baud_rate: u32,
    pub to_write: Arc<String>,
    pub data_bits: DruidDataBits,
    pub flow_control: DruidFlowControl,
    pub parity: DruidParity,
    pub stop_bits: DruidStopBits,
    pub protocol: Protocol,
    pub sender: Arc<UnboundedSender<GuiMessage>>,
    pub status: String,
}

pub struct PortNameLens;

impl Lens<AppData, String> for PortNameLens {
    fn with<R, F: FnOnce(&String) -> R>(&self, data: &AppData, f: F) -> R {
        f(&data.port_name)
    }

    fn with_mut<R, F: FnOnce(&mut String) -> R>(&self, data: &mut AppData, f: F) -> R {
        f(Arc::make_mut(&mut data.port_name))
    }
}

pub struct ToWriteLens;

impl Lens<AppData, String> for ToWriteLens {
    fn with<R, F: FnOnce(&String) -> R>(&self, data: &AppData, f: F) -> R {
        f(&data.to_write)
    }

    fn with_mut<R, F: FnOnce(&mut String) -> R>(&self, data: &mut AppData, f: F) -> R {
        f(Arc::make_mut(&mut data.to_write))
    }
}
