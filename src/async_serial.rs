use crate::{ByteDirection, GuiMessage};
use bytes::{BufMut, BytesMut};
use druid::{ExtEventSink, Selector};
use futures::{channel::mpsc::UnboundedReceiver, stream::StreamExt};
use futures_util::sink::SinkExt;
use std::{io::Error, time::Duration};
use tokio_serial::{DataBits, FlowControl, Parity, Serial, SerialPortSettings, StopBits};
use tokio_util::codec::{Decoder, Encoder};

pub const IO_DATA: Selector<(ByteDirection, Vec<u8>)> = Selector::new("event.io-data");
pub const IO_ERROR: Selector<&str> = Selector::new("event.io-error");

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

pub async fn serial_loop(
    event_sink: &ExtEventSink,
    mut receiver_gui: UnboundedReceiver<GuiMessage>,
) {
    let mut to_shutdown = false;
    while let Some(msg_gui) = receiver_gui.next().await {
        match msg_gui {
            GuiMessage::Open(config) => {
                let settings = SerialPortSettings {
                    baud_rate: config.baud_rate,
                    data_bits: DataBits::from(config.data_bits),
                    flow_control: FlowControl::from(config.flow_control),
                    parity: Parity::from(config.parity),
                    stop_bits: StopBits::from(config.stop_bits),
                    // timeout is not used cf tokio_serial
                    timeout: Duration::from_millis(1),
                };

                if let Ok(mut port) = Serial::from_path(config.port_name.as_str(), &settings) {
                    let (mut sender_data, mut receiver_data) = RawCodec::new().framed(port).split();

                    let mut error_reading = false;

                    loop {
                        tokio::select! {
                            msg_gui = receiver_gui.next() => {
                                match msg_gui {
                                    Some(GuiMessage::Open(config)) => {
                                        let settings = SerialPortSettings {
                                            baud_rate: config.baud_rate,
                                            data_bits: DataBits::from(config.data_bits),
                                            flow_control: FlowControl::from(config.flow_control),
                                            parity: Parity::from(config.parity),
                                            stop_bits: StopBits::from(config.stop_bits),
                                            // timeout is not used cf tokio_serial
                                            timeout: Duration::from_millis(1),
                                        };

                                        if let Ok(port_reconnect) = Serial::from_path(config.port_name.as_str(), &settings){
                                            port = port_reconnect;
                                            let tmp = RawCodec::new().framed(port).split();
                                            sender_data =  tmp.0;
                                            receiver_data = tmp.1;
                                            error_reading = false;
                                        };
                                    }
                                    Some(GuiMessage::Write(data)) => {
                                        let bytes = BytesMut::from(&data[..]);
                                        if let Err(_) = sender_data.send(bytes).await {
                                            event_sink.submit_command(IO_ERROR, "Cannot write data on the port", None)
                                                      .unwrap();
                                        } else {
                                            event_sink.submit_command(IO_DATA, (ByteDirection::Out, data.clone()), None).unwrap();
                                        }
                                    }
                                    Some(GuiMessage::Close) => break,
                                    Some(GuiMessage::Shutdown) => {
                                        to_shutdown = true;
                                        break;
                                    }
                                    _ => (),
                                };
                            }
                            data = receiver_data.next() => {
                                if let Some(Ok(data)) = data {
                                    event_sink.submit_command(IO_DATA, (ByteDirection::In, Vec::from(&data[..])), None).unwrap();
                                } else {
                                    if !error_reading {
                                        event_sink
                                            .submit_command(IO_ERROR, "Error while reading data", None)
                                            .unwrap();
                                        error_reading = true;
                                    }

                                    if let Ok(port_reconnect) = Serial::from_path(config.port_name.as_str(), &settings){
                                        port = port_reconnect;
                                        let tmp = RawCodec::new().framed(port).split();
                                        sender_data =  tmp.0;
                                        receiver_data = tmp.1;
                                        error_reading = false;
                                    };
                                }
                            }
                        }
                    }

                    if to_shutdown {
                        break;
                    }
                } else {
                    // TODO error message should be localized but LocalizedString is generic and currently I don't get why
                    // maybe send a label ?
                    // event_sink
                    //    .submit_command(IO_ERROR, Label::new(LocalizedString::new("Cannot open the port")), None)
                    //    .unwrap();
                    event_sink
                        .submit_command(IO_ERROR, "Cannot open the port", None)
                        .unwrap();
                }
            }
            GuiMessage::Shutdown => break,
            GuiMessage::Write(_) => event_sink
                .submit_command(IO_ERROR, "Cannot write data port not open", None)
                .unwrap(),
            _ => (),
        }
    }
}
