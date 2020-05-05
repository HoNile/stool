#![windows_subsystem = "windows"]

mod async_serial;
mod data;
mod ui;
mod widget_controller;

use crate::async_serial::{IO_DATA, IO_ERROR};
use crate::data::{
    AppData, DruidDataBits, DruidFlowControl, DruidParity, DruidStopBits, OpenMessage, Protocol,
};
use crate::ui::make_ui;
use druid::{
    AppLauncher, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    LocalizedString, PaintCtx, Selector, Size, UpdateCtx, Widget, WindowDesc,
};
use futures::channel::mpsc;
use std::{sync::Arc, thread};
use tokio::runtime::Runtime;

const OPEN_PORT: Selector = Selector::new("event.open-port");
const CLOSE_PORT: Selector = Selector::new("event.close-port");
const WRITE_PORT: Selector = Selector::new("event.write-port");

const MAX_VIEW_SIZE: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ByteDirection {
    Out,
    In,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuiMessage {
    Open(OpenMessage),
    Close,
    Write(Vec<u8>),
    Shutdown,
}

fn display_raw(io_data: &(ByteDirection, Vec<u8>), items: &mut Vec<String>, chunk_size: usize) {
    match io_data.0 {
        ByteDirection::Out => {
            let to_print = format!(
                "> {}",
                hex::encode_upper(&io_data.1)
                    .chars()
                    .enumerate()
                    .flat_map(|(i, c)| {
                        if i != 0 && i % 2 == 0 {
                            Some(' ')
                        } else {
                            None
                        }
                        .into_iter()
                        .chain(std::iter::once(c))
                    })
                    .collect::<String>()
            );
            if items.last().unwrap().is_empty() {
                items.last_mut().unwrap().push_str(&to_print);
            } else {
                items.push(to_print);
            }

            items.push("".to_string());
        }
        ByteDirection::In => {
            let new_data = hex::encode_upper(&io_data.1);
            let old_data: String = items.last().unwrap().split_ascii_whitespace().collect();

            let mut to_insert = old_data
                .chars()
                .chain(new_data.chars())
                .enumerate()
                .flat_map(|(i, c)| {
                    if i != 0 && i % 2 == 0 {
                        Some(' ')
                    } else {
                        None
                    }
                    .into_iter()
                    .chain(std::iter::once(c))
                });

            let mut collector = Vec::with_capacity((new_data.len() / chunk_size) + 2);
            loop {
                let tmp: String = (&mut to_insert).take(chunk_size).collect();
                if tmp == "" {
                    break;
                }
                collector.push(tmp);
            }

            for s in collector {
                let last_item = items.last_mut().unwrap();
                *last_item = s;
                items.push("".to_string());
            }
            if items[items.len() - 2].len() < chunk_size {
                items.pop();
            }
        }
    }
}

fn display_lines(io_data: &(ByteDirection, Vec<u8>), items: &mut Vec<String>, chunk_size: usize) {
    match io_data.0 {
        ByteDirection::Out => {
            let to_print = format!("> {}", String::from_utf8_lossy(&io_data.1));
            if items.last().unwrap().is_empty() {
                items.last_mut().unwrap().push_str(&to_print);
            } else {
                items.push(to_print);
            }

            items.push("".to_string());
        }
        ByteDirection::In => {
            if !items.last().unwrap().is_empty() {
                items.push("".to_string());
            }

            for line in String::from_utf8_lossy(&io_data.1).lines() {
                let line = line.to_string();
                let mut to_insert = line.chars();
                loop {
                    let items_len = items.len() - 1;
                    let tmp: String = (&mut to_insert).take(chunk_size).collect();
                    if tmp == "" {
                        break;
                    }
                    items[items_len].push_str(&tmp);
                    items.push("".to_string());
                }
            }
        }
    }
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        EventHandler {}
    }
}

impl Widget<AppData> for EventHandler {
    fn event(&mut self, _ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::Command(cmd) if cmd.selector == IO_DATA => {
                let io_data = cmd.get_object::<(ByteDirection, Vec<u8>)>().unwrap();
                let items = Arc::make_mut(&mut data.visual_items);

                // Init view to be able to run the same loop as if items was not empty
                if items.is_empty() {
                    items.push("".to_string());
                }

                match data.protocol {
                    Protocol::Raw => display_raw(io_data, items, data.line_size),
                    Protocol::Lines => display_lines(io_data, items, data.line_size),
                }

                // FIXME not efficient to do this on Vec
                let items_len = items.len();
                if items_len > MAX_VIEW_SIZE {
                    let keep_items = items.split_off(items_len - MAX_VIEW_SIZE);
                    *items = keep_items;
                }

                data.status = "".to_string();

                // TODO later I will want to store some raw_data to clear the visual and re-print ?
                // let raw_items = Arc::make_mut(&mut data.raw_items);
                // raw_items.append(&mut io_data.clone());
            }
            Event::Command(cmd) if cmd.selector == OPEN_PORT => {
                if let Ok(baud_rate) = data.baud_rate.parse::<u32>() {
                    data.sender
                        .unbounded_send(GuiMessage::Open(OpenMessage {
                            port_name: (*data.port_name).clone(),
                            baud_rate: baud_rate,
                            data_bits: data.data_bits,
                            flow_control: data.flow_control,
                            parity: data.parity,
                            stop_bits: data.stop_bits,
                            protocol: data.protocol,
                        }))
                        .unwrap();
                    data.status = "".to_string();
                } else {
                    data.status = "Incorrect Baudrate".to_string();
                }
            }
            Event::Command(cmd) if cmd.selector == CLOSE_PORT => {
                data.sender.unbounded_send(GuiMessage::Close).unwrap();
                data.status = "".to_string();
            }
            Event::Command(cmd) if cmd.selector == WRITE_PORT => match data.protocol {
                Protocol::Raw => {
                    let bytes: String = data.to_write.as_str().split_ascii_whitespace().collect();
                    if let Ok(bytes) = hex::decode(bytes) {
                        data.sender
                            .unbounded_send(GuiMessage::Write(bytes))
                            .unwrap();
                        data.status = "".to_string();
                    } else {
                        data.status = "Incorrect data doesn't respect protocol format".to_string();
                    }
                }
                Protocol::Lines => {
                    let bytes = data.to_write.as_bytes().to_owned();
                    data.sender
                        .unbounded_send(GuiMessage::Write(bytes))
                        .unwrap();
                    data.status = "".to_string();
                }
            },
            Event::Command(cmd) if cmd.selector == IO_ERROR => {
                // TODO should be a pop-up or something
                let error_msg = cmd.get_object::<&str>().unwrap();
                data.status = error_msg.to_string();
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &AppData, _: &Env) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppData, data: &AppData, _: &Env) {
        if !old_data.same(data) {
            ctx.request_layout();
            ctx.request_paint();
        }
    }

    fn layout(&mut self, _: &mut LayoutCtx, bc: &BoxConstraints, _: &AppData, _: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, _ctx: &mut PaintCtx, _data: &AppData, _env: &Env) {}
}

fn main() {
    let window = WindowDesc::new(make_ui)
        .title(LocalizedString::new("Serial tool").with_placeholder("Stool"))
        .with_min_size((166., 865.))
        .window_size((500., 865.));

    let launcher = AppLauncher::with_window(window);

    let event_sink = launcher.get_external_handle();

    let (sender, receiver) = mpsc::unbounded::<GuiMessage>();

    let runtime_join_handle = thread::spawn(move || {
        // Create the runtime
        let mut async_rt = Runtime::new().expect("runtime failed");
        async_rt.block_on(async_serial::serial_loop(&event_sink, receiver));
    });

    let sender_shutdown = sender.clone();

    launcher
        .launch(AppData {
            visual_items: Arc::new(vec![]),
            port_name: Arc::new("".to_string()),
            baud_rate: Arc::new("115200".to_string()),
            to_write: Arc::new("".to_string()),
            data_bits: DruidDataBits::Eight,
            flow_control: DruidFlowControl::None,
            parity: DruidParity::None,
            stop_bits: DruidStopBits::One,
            protocol: Protocol::Raw,
            sender: Arc::new(sender),
            raw_items: Arc::new(vec![]),
            line_size: 0,
            status: "".to_string(),
        })
        .expect("launch failed");

    sender_shutdown
        .unbounded_send(GuiMessage::Shutdown)
        .unwrap();
    runtime_join_handle.join().unwrap();
}
