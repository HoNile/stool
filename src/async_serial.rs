use bytes::{BufMut, BytesMut};
use druid::{ExtEventSink, Selector};
use futures::stream::StreamExt;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::{io::Error, thread};
use tokio_serial::Serial;
use tokio_serial::{DataBits, FlowControl, Parity, SerialPortSettings, StopBits};
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    DruidDataBits, DruidFlowControl, DruidParity, DruidStopBits, GuiMessage, OpenMessage, Protocol,
};

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

pub async fn serial_loop(event_sink: &ExtEventSink, receiver_gui: Receiver<GuiMessage>) {
    let (sender, receiver) = channel::<OpenMessage>();
    let (close_sender, close_receiver) = channel::<GuiMessage>();
    let (protocol_sender, protocol_receiver) = channel::<GuiMessage>();
    let (write_sender, write_receiver) = channel::<GuiMessage>();
    let (shutdown_sender, close_receiver) = channel::<GuiMessage>();

    let handle = thread::spawn(move || loop {
        let mut is_open = false;

        if let Ok(message) = receiver_gui.recv() {
            match message {
                GuiMessage::Open(open_msg) => {
                    if !is_open {
                        is_open = true;
                        sender.send(open_msg).unwrap();
                    };
                }
                GuiMessage::Close => {
                    close_sender.send(GuiMessage::Close).unwrap();
                    is_open = false;
                }
                GuiMessage::UpdateProtocol(protocol) => {
                    protocol_sender
                        .send(GuiMessage::UpdateProtocol(protocol))
                        .unwrap();
                }
                GuiMessage::Write(data) => {
                    write_sender.send(GuiMessage::Write(data)).unwrap();
                }
                GuiMessage::Shutdown => {
                    // FIXME hard to do like this
                    shutdown_sender.send(GuiMessage::Shutdown).unwrap();
                    break;
                }
            };
        }
    });

    if let Ok(mut config) = receiver.recv() {
        let mut settings = SerialPortSettings::default();

        settings.baud_rate = config.baud_rate.parse::<u32>().unwrap();

        settings.data_bits = match config.data_bits {
            DruidDataBits::Eight => DataBits::Eight,
            DruidDataBits::Seven => DataBits::Seven,
            DruidDataBits::Six => DataBits::Six,
            DruidDataBits::Five => DataBits::Five,
        };

        settings.flow_control = match config.flow_control {
            DruidFlowControl::Hardware => FlowControl::Hardware,
            DruidFlowControl::Software => FlowControl::Software,
            DruidFlowControl::None => FlowControl::None,
        };

        settings.parity = match config.parity {
            DruidParity::Even => Parity::Even,
            DruidParity::Odd => Parity::Odd,
            DruidParity::None => Parity::None,
        };

        settings.stop_bits = match config.stop_bits {
            DruidStopBits::One => StopBits::One,
            DruidStopBits::Two => StopBits::Two,
        };

        if let Ok(port) = Serial::from_path(config.port_name.as_str(), &settings) {
            let mut reader = RawCodec::new().framed(port);

            while let Some(data) = reader.next().await {
                if let Ok(data) = data {
                    if let Ok(new_protocol) = protocol_receiver.try_recv() {
                        match new_protocol {
                            GuiMessage::UpdateProtocol(new_protocol) => {
                                config.protocol = new_protocol;
                            }
                            _ => {
                                panic!();
                            }
                        }
                    }

                    match config.protocol {
                        Protocol::Raw => {
                            event_sink
                                .submit_command(ADD_ITEM, hex::encode_upper(data), None)
                                .unwrap();
                        }
                        Protocol::Lines => {
                            event_sink
                                .submit_command(
                                    ADD_ITEM,
                                    String::from_utf8_lossy(&data[..]).to_string(),
                                    None,
                                )
                                .unwrap();
                        }
                    }
                } else {
                    break;
                }
            }
        }
    }

    handle.join().unwrap();
}
