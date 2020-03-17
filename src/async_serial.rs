use bytes::{BufMut, BytesMut};
use druid::{ExtEventSink, Selector};
use futures::stream::StreamExt;
use std::sync::mpsc::Receiver;
use std::{io::Error, str};
use tokio::runtime::Runtime;
use tokio_serial::Serial;
use tokio_serial::{DataBits, FlowControl, Parity, SerialPortSettings, StopBits};
use tokio_util::codec::{Decoder, Encoder};

use crate::{DruidDataBits, DruidFlowControl, DruidParity, DruidStopBits, GuiMessage, Protocol};

pub const ADD_ITEM: Selector = Selector::new("event.add-item");

pub struct RawCodec;

impl RawCodec {
    pub fn new() -> Self {
        RawCodec {}
    }
}

impl Decoder for RawCodec {
    type Item = BytesMut;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<BytesMut>, Error> {
        let data = buf.split_to(buf.len());
        if data.len() > 0 {
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for RawCodec {
    type Item = BytesMut;
    type Error = std::io::Error;

    fn encode(&mut self, tc_data: BytesMut, buf: &mut BytesMut) -> Result<(), Error> {
        buf.reserve(tc_data.len());
        buf.put_slice(&tc_data[..]);

        Ok(())
    }
}

pub async fn close() {
    todo!();
}

pub async fn update_protocol(protocol: Protocol) {
    todo!();
}

pub async fn write(data: Vec<u8>) {
    todo!();
}

pub async fn shutdown() {
    todo!();
}

pub async fn serial_loop(
    event_sink: &ExtEventSink,
    settings: &SerialPortSettings,
    name: &str,
    protocol: Protocol,
) {
    if let Ok(port) = Serial::from_path(name, &settings) {
        let mut reader = RawCodec::new().framed(port);

        while let Some(data) = reader.next().await {
            if let Ok(data) = data {
                match protocol {
                    Protocol::Raw => {
                        event_sink
                            .submit_command(ADD_ITEM, hex::encode_upper(data), None)
                            .unwrap();
                    }
                    Protocol::Lines => {
                        let to_send = match str::from_utf8(&data[..]) {
                            Ok(data) => data,
                            Err(_) => "?",
                        };
                        event_sink
                            .submit_command(ADD_ITEM, to_send.to_string(), None)
                            .unwrap();
                    }
                }
            } else {
                break;
            }
        }
    }
}

pub fn runtime(event_sink: ExtEventSink, receiver: Receiver<GuiMessage>) {
    let mut settings = SerialPortSettings::default();
    // Create the runtime
    let mut async_rt = Runtime::new().unwrap();

    let mut is_open = false;

    loop {
        if let Ok(message) = receiver.recv() {
            match message {
                GuiMessage::Open(open_msg) => {
                    if !is_open {
                        is_open = true;
                        let name = open_msg.port_name.as_str();

                        settings.baud_rate = open_msg.baud_rate.parse::<u32>().unwrap();

                        settings.data_bits = match open_msg.data_bits {
                            DruidDataBits::Eight => DataBits::Eight,
                            DruidDataBits::Seven => DataBits::Seven,
                            DruidDataBits::Six => DataBits::Six,
                            DruidDataBits::Five => DataBits::Five,
                        };

                        settings.flow_control = match open_msg.flow_control {
                            DruidFlowControl::Hardware => FlowControl::Hardware,
                            DruidFlowControl::Software => FlowControl::Software,
                            DruidFlowControl::None => FlowControl::None,
                        };

                        settings.parity = match open_msg.parity {
                            DruidParity::Even => Parity::Even,
                            DruidParity::Odd => Parity::Odd,
                            DruidParity::None => Parity::None,
                        };

                        settings.stop_bits = match open_msg.stop_bits {
                            DruidStopBits::One => StopBits::One,
                            DruidStopBits::Two => StopBits::Two,
                        };

                        async_rt.block_on(serial_loop(
                            &event_sink,
                            &settings,
                            name,
                            open_msg.protocol,
                        ));
                    };
                }
                GuiMessage::Close => {
                    async_rt.spawn(close());
                }
                GuiMessage::UpdateProtocol(protocol) => {
                    async_rt.spawn(update_protocol(protocol));
                }
                GuiMessage::Write(data) => {
                    async_rt.spawn(write(data));
                }
                GuiMessage::Shutdown => {
                    async_rt.spawn(shutdown());
                    break;
                }
            };
        }
    }
}
