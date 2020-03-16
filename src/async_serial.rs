use bytes::{BufMut, BytesMut};
use druid::{ExtEventSink, Selector};
use futures::stream::StreamExt;
use std::{io::Error, slice, thread, time::Duration};
use tokio_serial::{Serial, SerialPortSettings};
use tokio_util::codec::{Decoder, Encoder};

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

pub async fn serial_loop(event_sink: &ExtEventSink, settings: &SerialPortSettings, name: &str) {
    // TODO remove begin test
    let test: Vec<u8> = vec![
        0x10, 0xA5, 0x25, 0x00, 0x09, 0x10, 0xA5, 0x25, 0x00, 0x09, 0x10, 0xA5, 0x25, 0x00, 0x10,
        0xA5, 0x25, 0x00, 0x09, 0x10, 0xA5, 0x25, 0x00, 0x09, 0x10, 0xA5, 0x25, 0x00, 0x10, 0xA5,
        0x25, 0x00, 0x09, 0x10, 0xA5, 0x25, 0x00, 0x09, 0x10, 0xA5, 0x25, 0x00, 0x10, 0xA5, 0x25,
        0x00, 0x09, 0x10, 0xA5, 0x25, 0x00, 0x09, 0x10, 0xA5, 0x25, 0x00, 0x10, 0xA5, 0x25, 0x00,
        0x09, 0x10, 0xA5, 0x25, 0x00, 0x09, 0x10, 0xA5, 0x25, 0x00, 0x10, 0xA5, 0x25, 0x00, 0x09,
        0x10, 0xA5, 0x25, 0x00, 0x09, 0x10, 0xA5, 0x25, 0x00, 0x10, 0xA5, 0x25, 0x00, 0x09, 0x10,
        0xA5, 0x25, 0x00, 0x09, 0x10, 0xA5, 0x25, 0x00, 0x10, 0xA5, 0x25, 0x00, 0x09, 0x10, 0xA5,
    ];
    for data in test.iter() {
        event_sink
            .submit_command(ADD_ITEM, hex::encode_upper(slice::from_ref(data)), None)
            .unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    // end test

    if let Ok(port) = Serial::from_path(name, &settings) {
        let mut reader = RawCodec::new().framed(port);

        while let Some(data) = reader.next().await {
            if let Ok(data) = data {
                event_sink
                    .submit_command(ADD_ITEM, hex::encode_upper(data), None)
                    .unwrap();
            } else {
                break;
            }
        }
        println!("The end !");
    }
}
