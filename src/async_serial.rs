use bytes::{BufMut, BytesMut};
use druid::{ExtEventSink, Selector};
use futures::{channel::mpsc, stream::StreamExt};
use futures_util::sink::SinkExt;
use std::{
    io::Error,
    sync::{atomic::AtomicBool, atomic::Ordering, mpsc::Receiver, Arc},
    thread,
    time::Duration,
};
use tokio::time;
use tokio_serial::{DataBits, FlowControl, Parity, Serial, SerialPortSettings, StopBits};
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    DruidDataBits, DruidFlowControl, DruidParity, DruidStopBits, GuiMessage, OpenMessage, Protocol,
};

pub const READ_ITEM: Selector = Selector::new("event.read-item");
pub const WRITE_ITEM: Selector = Selector::new("event.write-item");

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
    let (close_sender, mut close_receiver) = mpsc::unbounded::<()>();
    let (protocol_sender, mut protocol_receiver) = mpsc::unbounded::<Protocol>();
    let (write_sender, mut write_receiver) = mpsc::unbounded::<Vec<u8>>();
    let (shutdown_sender, mut shutdown_receiver) = mpsc::unbounded::<()>();

    let is_open = Arc::new(AtomicBool::new(false));

    let cl_is_open = is_open.clone();
    let handle = thread::spawn(move || {
        loop {
            if let Ok(message) = receiver_gui.recv() {
                let local_is_open = cl_is_open.load(Ordering::SeqCst);
                match message {
                    GuiMessage::Open(open_msg) => {
                        if !local_is_open {
                            sender.unbounded_send(open_msg).unwrap();
                        };
                    }
                    GuiMessage::Close => {
                        if local_is_open {
                            close_sender.unbounded_send(()).unwrap();
                        }
                    }
                    GuiMessage::UpdateProtocol(protocol) => {
                        if local_is_open {
                            protocol_sender.unbounded_send(protocol).unwrap();
                        }
                    }
                    GuiMessage::Write(data) => {
                        if local_is_open {
                            write_sender.unbounded_send(data).unwrap();
                        }
                    }
                    GuiMessage::Shutdown => {
                        // FIXME what happens currently if no port open ?
                        shutdown_sender.unbounded_send(()).unwrap();
                        break;
                    }
                };
            }
        }
    });

    let mut to_shutdown = false;
    while let Some(mut config) = receiver.next().await {
        /*loop {
        let (config, hello) = (receiver.next().await, to_shutdown);
        if !hello{
            if let Some(config) = config {

            }
        }*/
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
            is_open.store(true, Ordering::SeqCst);
            let (mut writer, mut reader) = RawCodec::new().framed(port).split();

            // GUI didn't like to received a lot of small change really fast so I update it every 5 milliseconds if needed
            let mut refresh_gui = time::interval(Duration::from_millis(5));
            let mut accumulate_data = String::new();
            loop {
                tokio::select! {
                    new_protocol = protocol_receiver.next() => {
                        if let Some(new_protocol) = new_protocol {
                            config.protocol = new_protocol;
                        }
                    }
                    data = reader.next() => {
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
                            break;
                        }
                    }
                    data = write_receiver.next() => {
                        if let Some(data) = data {
                            // FIXME need new line somewhere
                            accumulate_data.push_str(format!("> {}", hex::encode_upper(&data)).as_str());
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
                    _ = refresh_gui.tick() => {
                        if !accumulate_data.is_empty() {
                            event_sink.submit_command(WRITE_ITEM, accumulate_data.clone(), None).unwrap();
                            accumulate_data.clear();
                        }
                    }
                }
            }

            is_open.store(false, Ordering::SeqCst);
            if to_shutdown {
                break;
            }
        }
    }

    handle.join().unwrap();
}
