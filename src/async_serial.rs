use crate::{ByteDirection, GuiMessage};
use bytes::{BufMut, BytesMut};
use druid::{ExtEventSink, Selector};
use futures::{channel::mpsc::UnboundedReceiver, stream::StreamExt};
use futures_util::sink::SinkExt;
use std::{io::Error, time::Duration};
use tokio::time;
use tokio_serial::{DataBits, FlowControl, Parity, Serial, SerialPortSettings, StopBits};
use tokio_util::codec::{Decoder, Encoder};

pub const IO_DATA: Selector = Selector::new("event.io-data");
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

                if let Ok(port) = Serial::from_path(config.port_name.as_str(), &settings) {
                    let (mut sender_data, mut receiver_data) = RawCodec::new().framed(port).split();

                    // WARNING all the system refresh_gui/accumulate_data make it is easy to make mistake
                    // GUI didn't like to received a lot of small change really fast so I update it every 5 milliseconds if needed
                    let mut refresh_gui = time::interval(Duration::from_millis(5));
                    let mut accumulate_data = Vec::<(ByteDirection, Vec<u8>)>::new();
                    // changing refresh_gui period make the next await be ready so I need this bool to not update it in loop
                    let mut fast_refresh = true;

                    loop {
                        tokio::select! {
                            msg_gui = receiver_gui.next() => {
                                match msg_gui {
                                    Some(GuiMessage::Write(data)) => {
                                        accumulate_data.push((ByteDirection::Out, data.clone()));
                                        let bytes = BytesMut::from(&data[..]);
                                        if let Err(_) = sender_data.send(bytes).await {
                                            event_sink.submit_command(IO_ERROR, "Cannot write data on the port", None)
                                                      .unwrap();
                                        }
                                        if !fast_refresh {
                                            refresh_gui = time::interval(Duration::from_millis(5));
                                            fast_refresh = true;
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
                                    if let Some(last) = accumulate_data.last_mut() {
                                        if last.0 == ByteDirection::In {
                                            last.1.extend_from_slice(&data);
                                        } else {
                                            accumulate_data.push((ByteDirection::In, Vec::from(&data[..])));
                                        }
                                    } else {
                                        accumulate_data.push((ByteDirection::In, Vec::from(&data[..])));
                                    }
                                    if !fast_refresh {
                                        refresh_gui = time::interval(Duration::from_millis(5));
                                        fast_refresh = true;
                                    }
                                } else {
                                    // FIXME this should not be send in loop example port disconected
                                    // TODO check if reconnect the port is possible
                                    event_sink
                                        .submit_command(IO_ERROR, "Error while reading data", None)
                                        .unwrap();
                                }
                            }
                            _ = refresh_gui.tick() => {
                                if !accumulate_data.is_empty() {
                                    event_sink.submit_command(IO_DATA, accumulate_data.clone(), None).unwrap();
                                    accumulate_data.clear();
                                } else {
                                    // Avoid working if nothing to do
                                    if fast_refresh {
                                        refresh_gui = time::interval(Duration::from_secs(86_400));
                                        fast_refresh = false
                                    }
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
            _ => (),
        }
    }
}
