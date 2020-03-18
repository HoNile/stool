use bytes::{BufMut, BytesMut};
use druid::{ExtEventSink, Selector};
use futures::{channel::mpsc, stream::StreamExt};
use futures_util::sink::SinkExt;
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
    let (sender, mut receiver) = mpsc::unbounded::<OpenMessage>();
    let (close_sender, mut close_receiver) = mpsc::unbounded::<GuiMessage>();
    let (protocol_sender, mut protocol_receiver) = mpsc::unbounded::<Protocol>();
    let (write_sender, mut write_receiver) = mpsc::unbounded::<Vec<u8>>();
    let (shutdown_sender, mut shutdown_receiver) = mpsc::unbounded::<GuiMessage>();

    let handle = thread::spawn(move || loop {
        let mut is_open = false;

        if let Ok(message) = receiver_gui.recv() {
            match message {
                GuiMessage::Open(open_msg) => {
                    if !is_open {
                        is_open = true;
                        sender.unbounded_send(open_msg).unwrap();
                        //tokio::spawn(async { sender.send(open_msg).await.unwrap() });
                    };
                }
                GuiMessage::Close => {
                    close_sender.unbounded_send(GuiMessage::Close).unwrap();
                    is_open = false;
                }
                GuiMessage::UpdateProtocol(protocol) => {
                    protocol_sender.unbounded_send(protocol).unwrap();
                }
                GuiMessage::Write(data) => {
                    write_sender.unbounded_send(data).unwrap();
                }
                GuiMessage::Shutdown => {
                    shutdown_sender
                        .unbounded_send(GuiMessage::Shutdown)
                        .unwrap();
                    break;
                }
            };
        }
    });

    let mut to_shutdown = false;
    while let Some(mut config) = receiver.next().await {
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
            let (mut writer, mut reader) = RawCodec::new().framed(port).split();

            loop {
                tokio::select! {
                    data = reader.next() => {
                        if let Some(Ok(data)) = data {
                            if let Some(new_protocol) = protocol_receiver.next().await {
                                config.protocol = new_protocol;
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
                    data = write_receiver.next() => {
                        if let Some(data) = data {
                            let mut bytes = BytesMut::with_capacity(data.len());
                            bytes.put(&data[..]);
                            writer.send(bytes).await.unwrap();
                        }
                    }
                    msg = close_receiver.next() => {
                        if let Some(_) = msg {
                            break;
                        }
                    }
                    msg = shutdown_receiver.next() => {
                        if let Some(_) = msg {
                            to_shutdown = true;
                            break;
                        }
                    }
                }
            }

            if to_shutdown {
                break;
            }
        }
    }

    handle.join().unwrap();
}
