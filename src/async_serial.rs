use bytes::{BufMut, BytesMut};
use druid::{ExtEventSink, Selector};
use futures::stream::StreamExt;
use std::{io::Error, str};
use tokio_serial::{Serial, SerialPortSettings};
use tokio_util::codec::{Decoder, Encoder};

use crate::Protocol;

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
        println!("The end !");
    }
}
