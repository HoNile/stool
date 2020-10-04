#![windows_subsystem = "windows"]

mod async_serial;
mod data;
mod ui;

use crate::async_serial::{IO_DATA, IO_ERROR};
use crate::data::{
    AppData, DruidDataBits, DruidFlowControl, DruidParity, DruidStopBits, OpenMessage, Protocol,
};
use crate::ui::make_ui;
use druid::piet::TextStorage;
use druid::platform_menus;
use druid::text::RichText;
use druid::{
    AppLauncher, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    LocalizedString, MenuDesc, PaintCtx, Selector, Size, UpdateCtx, Widget, WindowDesc,
};
use futures::channel::mpsc;
use std::{sync::Arc, thread};
use tokio::runtime::Runtime;

const OPEN_PORT: Selector = Selector::new("event.open-port");
const CLOSE_PORT: Selector = Selector::new("event.close-port");
const WRITE_PORT: Selector = Selector::new("event.write-port");

const MAX_VIEW_SIZE: usize = 1024 * 512;

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

fn display_raw(io_data: &(ByteDirection, Vec<u8>), output: &mut RichText) {
    match io_data.0 {
        ByteDirection::Out => {
            let mut to_print = format!(
                "> {}\n",
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

            let curr_output = output.as_str();
            to_print = format!("{}{}", curr_output, to_print);

            if curr_output.is_empty() {
                *output = RichText::new(to_print.into());
            } else if curr_output.chars().rev().take(1).next() == Some('\n') {
                *output = RichText::new(to_print.into());
            } else {
                to_print = format!("\n{}", to_print);
                *output = RichText::new(to_print.into());
            }
        }
        ByteDirection::In => {
            let new_data = hex::encode_upper(&io_data.1);
            let output_last_line: Option<&str> = output.as_str().lines().rev().take(1).next();

            if let Some(last_line) = output_last_line {
                let old_data: String = last_line.split_ascii_whitespace().collect();
                let to_insert: String = old_data
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
                    })
                    .collect();
                let mut new_output = "".to_string();
                let line_number = output.as_str().lines().count();
                for line in output.as_str().lines().take(line_number - 1) {
                    new_output = format!("{}\n{}", new_output, line);
                }
                new_output = format!("{}\n{}", new_output, to_insert);
                *output = RichText::new(new_output.into());
            } else {
                let to_insert: String = new_data
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
                    .collect();
                *output = RichText::new(to_insert.into());
            }
        }
    }
}

fn display_text(io_data: &(ByteDirection, Vec<u8>), output: &mut RichText) {
    match io_data.0 {
        ByteDirection::Out => {
            let mut to_print = format!("> {}\n", String::from_utf8_lossy(&io_data.1));

            let curr_output = output.as_str();
            to_print = format!("{}{}", curr_output, to_print);

            if curr_output.is_empty() {
                *output = RichText::new(to_print.into());
            } else if curr_output.chars().rev().take(1).next() == Some('\n') {
                *output = RichText::new(to_print.into());
            } else {
                to_print = format!("\n{}", to_print);
                *output = RichText::new(to_print.into());
            }
        }
        ByteDirection::In => {
            let to_print = format!("{}{}", output.as_str(), String::from_utf8_lossy(&io_data.1));
            *output = RichText::new(to_print.into());
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
            Event::Command(cmd) if cmd.is(IO_DATA) => {
                let io_data = cmd.get_unchecked(IO_DATA);

                match data.protocol {
                    Protocol::Raw => display_raw(io_data, &mut data.output),
                    Protocol::Text => display_text(io_data, &mut data.output),
                }

                // FIXME not efficient to do this on Vec/String
                let output_len = data.output.as_str().len();
                if output_len > MAX_VIEW_SIZE {
                    let keep_output = data
                        .output
                        .as_str()
                        .to_string()
                        .split_off(output_len - MAX_VIEW_SIZE);
                    data.output = RichText::new(keep_output.into());
                }

                data.status = "".to_string();
            }
            Event::Command(cmd) if cmd.is(OPEN_PORT) => {
                data.sender
                    .unbounded_send(GuiMessage::Open(OpenMessage {
                        port_name: (*data.port_name).clone(),
                        baud_rate: data.baud_rate,
                        data_bits: data.data_bits,
                        flow_control: data.flow_control,
                        parity: data.parity,
                        stop_bits: data.stop_bits,
                        protocol: data.protocol,
                    }))
                    .unwrap();
                data.status = "".to_string();
            }
            Event::Command(cmd) if cmd.is(CLOSE_PORT) => {
                data.sender.unbounded_send(GuiMessage::Close).unwrap();
                data.status = "".to_string();
            }
            Event::Command(cmd) if cmd.is(WRITE_PORT) => match data.protocol {
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
                Protocol::Text => {
                    let bytes = data.to_write.as_bytes().to_owned();
                    data.sender
                        .unbounded_send(GuiMessage::Write(bytes))
                        .unwrap();
                    data.status = "".to_string();
                }
            },
            Event::Command(cmd) if cmd.is(IO_ERROR) => {
                // TODO should be a pop-up or something
                let error_msg = cmd.get_unchecked(IO_ERROR);
                data.status = error_msg.to_string();
            }
            _ => {}
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
    // FIXME menu is temporary, it's needed to copy/paste to work
    let window = WindowDesc::new(make_ui)
        .menu(
            MenuDesc::empty()
                .append(platform_menus::common::cut())
                .append(platform_menus::common::copy())
                .append(platform_menus::common::paste()),
        )
        .title(LocalizedString::new("Serial tool").with_placeholder("Stool"))
        .with_min_size((170., 930.))
        .window_size((500., 930.));

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
            output: RichText::new("".into()),
            port_name: Arc::new("".to_string()),
            baud_rate: 115_200,
            to_write: Arc::new("".to_string()),
            data_bits: DruidDataBits::Eight,
            flow_control: DruidFlowControl::None,
            parity: DruidParity::None,
            stop_bits: DruidStopBits::One,
            protocol: Protocol::Raw,
            sender: Arc::new(sender),
            status: "".to_string(),
        })
        .expect("launch failed");

    sender_shutdown
        .unbounded_send(GuiMessage::Shutdown)
        .unwrap();
    runtime_join_handle.join().unwrap();
}
