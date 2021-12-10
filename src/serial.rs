use crate::{data::OpenMessage, GuiMessage};
use bytes::{BufMut, Bytes, BytesMut};
use druid::{ExtEventError, ExtEventSink, Selector, Target};
use futures::{channel::mpsc::UnboundedReceiver, stream::StreamExt};
use futures_util::sink::SinkExt;
use std::io::Error;
use tokio_serial::{
    DataBits, FlowControl, Parity, SerialPortBuilder, SerialPortBuilderExt, SerialStream, StopBits,
};
use tokio_util::codec::{Decoder, Encoder};

pub const IO_DATA: Selector<(ByteDirection, Bytes)> = Selector::new("event.io-data");
pub const IO_ERROR: Selector<&str> = Selector::new("event.io-error");

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ByteDirection {
    Out,
    In,
}
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
        if buf.len() > 0 {
            Ok(Some(buf.clone()))
        } else {
            Ok(None)
        }
    }
}

impl Encoder<Bytes> for RawCodec {
    type Error = std::io::Error;

    fn encode(&mut self, data: Bytes, buf: &mut BytesMut) -> Result<(), Error> {
        buf.reserve(data.len());
        buf.put(data);

        Ok(())
    }
}

fn port_from_config(config: &OpenMessage) -> SerialPortBuilder {
    tokio_serial::new(config.port_name.as_str(), config.baud_rate)
        .baud_rate(config.baud_rate)
        .data_bits(DataBits::from(config.data_bits))
        .flow_control(FlowControl::from(config.flow_control))
        .parity(Parity::from(config.parity))
        .stop_bits(StopBits::from(config.stop_bits))
}

pub async fn serial_loop(
    event_sink: ExtEventSink,
    mut receiver_gui: UnboundedReceiver<GuiMessage>,
) -> Result<(), ExtEventError> {
    let send_err_gui = |data| event_sink.submit_command(IO_ERROR, data, Target::Global);

    while let Some(msg_gui) = receiver_gui.next().await {
        match msg_gui {
            GuiMessage::Open(config) => {
                let build_port = port_from_config(&config);
                if let Ok(port) = build_port.open_native_async() {
                    open_loop(&event_sink, &mut receiver_gui, port, &config).await?;
                } else {
                    send_err_gui("Cannot open the port")?;
                }
            }
            GuiMessage::Write(_) => send_err_gui("Cannot write data port not open")?,
            GuiMessage::Close => (),
        }
    }
    Ok(())
}

async fn open_loop(
    event_sink: &ExtEventSink,
    receiver_gui: &mut UnboundedReceiver<GuiMessage>,
    mut port: SerialStream,
    config: &OpenMessage,
) -> Result<(), ExtEventError> {
    let send_err_gui = |data| event_sink.submit_command(IO_ERROR, data, Target::Global);
    let send_data_gui = |dir, data| event_sink.submit_command(IO_DATA, (dir, data), Target::Global);
    let (mut sender_data, mut receiver_data) = RawCodec::new().framed(port).split();
    let mut error_reading = false;

    loop {
        tokio::select! {
            msg_gui = receiver_gui.next() => {
                match msg_gui {
                    Some(GuiMessage::Open(config)) => {
                        let build_port = port_from_config(&config);

                        if let Ok(new_port) = build_port.open_native_async() {
                            port = new_port;
                            let tmp = RawCodec::new().framed(port).split();
                            sender_data =  tmp.0;
                            receiver_data = tmp.1;
                            error_reading = false;
                        } else {
                            send_err_gui( "Cannot open the port")?;
                        }
                    }
                    Some(GuiMessage::Write(data)) => {
                        if let Err(_) = sender_data.send(data.clone()).await {
                            send_err_gui( "Cannot write data on the port")?;
                        } else {
                            send_data_gui(ByteDirection::Out, data)?;
                        }
                    }
                    Some(GuiMessage::Close) => return Ok(()),
                    None => return Err(ExtEventError),
                };
            }
            data = receiver_data.next() => {
                if let Some(Ok(data)) = data {
                    send_data_gui(ByteDirection::In, data.freeze())?;
                } else {
                    if !error_reading {
                        send_err_gui( "Error while reading data")?;
                        error_reading = true;
                    }

                    let build_port = port_from_config(&config);
                    if let Ok(port_reconnect) = build_port.open_native_async() {
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
}
