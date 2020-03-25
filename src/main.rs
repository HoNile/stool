mod async_serial;
mod data;
mod ui;

use crate::async_serial::{IO_ERROR, READ_ITEM, WRITE_ITEM};
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

/*#[derive(Debug, Clone, Copy)]
pub enum DataType {
    Write,
    Read,
}*/

#[derive(Debug, Clone, PartialEq)]
pub enum GuiMessage {
    Open(OpenMessage),
    Close,
    UpdateProtocol(Protocol),
    Write(Vec<u8>),
    Shutdown,
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
            // FIXME mix READ and WRITE item is not clean currently
            Event::Command(cmd) if cmd.selector == WRITE_ITEM => {
                let items = Arc::make_mut(&mut data.visual_items);
                if items.is_empty() {
                    items.push(cmd.get_object::<String>().unwrap().clone());
                    items.push("".to_string());
                } else {
                    let items_idx = items.len() - 1;
                    if items[items_idx] == "" {
                        items[items_idx] = cmd.get_object::<String>().unwrap().clone();
                        items.push("".to_string());
                    } else {
                        items.push(cmd.get_object::<String>().unwrap().clone());
                        items.push("".to_string());
                    }
                }

                // FIXME not efficient to do this on Vec
                while items.len() > MAX_VIEW_SIZE {
                    items.remove(0);
                }
            }
            Event::Command(cmd) if cmd.selector == READ_ITEM => {
                let items = Arc::make_mut(&mut data.visual_items);

                if items.is_empty() {
                    items.push("".to_string());
                }

                // Update last item until it grow to a certain size then add a new one
                let mut items_idx = items.len() - 1;
                let chunk_size: usize = 45;

                match data.protocol {
                    Protocol::Lines => {
                        for line in cmd.get_object::<String>().unwrap().lines() {
                            let item_idx_len = items[items_idx].len();
                            if item_idx_len > 0 {
                                items[items_idx].insert_str(item_idx_len - 1, line);
                            } else {
                                items[items_idx].insert_str(0, line);
                            }
                            items.push("".to_string());
                            items_idx += 1;
                        }
                    }
                    Protocol::Raw => {
                        let old_data: String = items[items_idx].split_ascii_whitespace().collect();
                        let new_data = cmd.get_object::<String>().unwrap();

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

                        items[items_idx].clear();
                        for s in collector {
                            items[items_idx] = s;
                            if items[items_idx].len() == chunk_size {
                                items.push("".to_string());
                                items_idx += 1;
                            }
                        }
                    }
                }

                // FIXME not efficient to do this on Vec
                while items.len() > MAX_VIEW_SIZE {
                    items.remove(0);
                }
            }
            Event::Command(cmd) if cmd.selector == OPEN_PORT => {
                // FIXME protocol should be update on click in its RadioGroup
                data.sender
                    .unbounded_send(GuiMessage::UpdateProtocol(data.protocol))
                    .unwrap();

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
                        .unwrap()
                } else {
                    println!("Incorrect Baudrate");
                }
            }
            Event::Command(cmd) if cmd.selector == CLOSE_PORT => {
                data.sender.unbounded_send(GuiMessage::Close).unwrap()
            }
            Event::Command(cmd) if cmd.selector == WRITE_PORT => {
                match data.protocol {
                    Protocol::Raw => {
                        let bytes: String =
                            data.to_write.as_str().split_ascii_whitespace().collect();
                        if let Ok(bytes) = hex::decode(bytes) {
                            data.sender
                                .unbounded_send(GuiMessage::Write(bytes))
                                .unwrap();
                        } else {
                            // TODO
                            println!("Incorrect data doesn't respect protocol format");
                        }
                    }
                    Protocol::Lines => {
                        let bytes = data.to_write.as_bytes().to_owned();
                        data.sender
                            .unbounded_send(GuiMessage::Write(bytes))
                            .unwrap();
                    }
                }
            }
            Event::Command(cmd) if cmd.selector == IO_ERROR => {
                // TODO should be a pop-up or something
                let error_msg = cmd.get_object::<&str>().unwrap();
                println!("{}", error_msg);
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
        .with_min_size((166., 850.))
        .window_size((500., 850.));

    let launcher = AppLauncher::with_window(window);

    let event_sink = launcher.get_external_handle();

    let (sender, receiver) = mpsc::unbounded::<GuiMessage>();

    thread::spawn(move || {
        // Create the runtime
        let mut async_rt = Runtime::new().expect("runtime failed");
        async_rt.block_on(async_serial::serial_loop(&event_sink, receiver));
    });

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
            //raw_items: Arc::new(vec![]),
        })
        .expect("launch failed");
}
