use crate::{GuiMessage, Protocol};
use bytes::{BufMut, BytesMut};
use druid::{ExtEventSink, Selector};
use futures::{channel::mpsc::UnboundedReceiver, stream::StreamExt};
use futures_util::sink::SinkExt;
use std::{io::Error, time::Duration};
use tokio::time;
use tokio_serial::{DataBits, FlowControl, Parity, Serial, SerialPortSettings, StopBits};
use tokio_util::codec::{Decoder, Encoder};

pub const READ_ITEM: Selector = Selector::new("event.read-item");
pub const WRITE_ITEM: Selector = Selector::new("event.write-item");
pub const IO_ERROR: Selector = Selector::new("event.io-error");

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
            GuiMessage::Open(mut config) => {
                let settings = SerialPortSettings {
                    baud_rate: config.baud_rate,
                    data_bits: DataBits::from(config.data_bits),
                    flow_control: FlowControl::from(config.flow_control),
                    parity: Parity::from(config.parity),
                    stop_bits: StopBits::from(config.stop_bits),
                    // timeout is not used cf tokio_serial
                    timeout: Duration::from_millis(1),
                };

                if let Ok(port) = Serial::from_path(config.port_name.as_str(), &settings) {
                    let (mut writer_data, mut reader_data) = RawCodec::new().framed(port).split();

                    // GUI didn't like to received a lot of small change really fast so I update it every 5 milliseconds if needed
                    let mut refresh_gui = time::interval(Duration::from_millis(5));
                    let mut accumulate_data = String::new();

                    loop {
                        tokio::select! {
                            msg_gui = receiver_gui.next() => {
                                match msg_gui {
                                    Some(GuiMessage::UpdateProtocol(new_protocol)) => {
                                        config.protocol = new_protocol
                                    }
                                    Some(GuiMessage::Write(data)) => {
                                        // FIXME need new line somewhere
                                        accumulate_data.push_str(format!("> {}", hex::encode_upper(&data)).as_str());
                                        let bytes = BytesMut::from(&data[..]);
                                        if let Err(_) = writer_data.send(bytes).await {
                                            event_sink.submit_command(IO_ERROR, "Cannot write data on the port", None)
                                                      .unwrap();
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
                            data = reader_data.next() => {
                                if let Some(Ok(data)) = data {
                                    match config.protocol {
                                        Protocol::Raw => {
                                            accumulate_data.push_str(hex::encode_upper(data).as_str());
                                        }
                                        Protocol::Lines => {
                                            // note there is still something strange in druid/piet but since last update didn't crash
                                            // TODO report a issue
                                            let to_send = String::from_utf8_lossy(&data).to_string();
                                            accumulate_data.push_str(to_send.as_str());
                                        }
                                    }
                                } else {
                                    event_sink
                                        .submit_command(IO_ERROR, "Error while reading data", None)
                                        .unwrap();
                                }
                            }
                            _ = refresh_gui.tick() => {
                                if !accumulate_data.is_empty() {
                                    event_sink.submit_command(WRITE_ITEM, accumulate_data.clone(), None).unwrap();
                                    accumulate_data.clear();
                                }
                            }
                        }
                    }

                    if to_shutdown {
                        break;
                    }
                } else {
                    // TODO error message should be localized but LocalizedString is generic and currently I don't get why
                    event_sink
                        .submit_command(IO_ERROR, "Cannot open the port", None)
                        .unwrap();
                }
            }
            GuiMessage::Shutdown => break,
            _ => (),
        }
    }
}
