mod async_serial;
mod stool_ui;

use async_serial::{READ_ITEM, WRITE_ITEM};
use druid::{
    AppLauncher, BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, LocalizedString, PaintCtx, Selector, Size, UpdateCtx, Widget, WindowDesc,
};
use std::{
    sync::{
        mpsc::{channel, Sender},
        Arc,
    },
    thread,
};
use stool_ui::{
    DruidDataBits, DruidFlowControl, DruidParity, DruidStopBits, GuiMessage, OpenMessage, Protocol,
};
use tokio::runtime::Runtime;

const OPEN_PORT: Selector = Selector::new("event.open-port");
const CLOSE_PORT: Selector = Selector::new("event.close-port");
const WRITE_PORT: Selector = Selector::new("event.write-port");

pub struct EventHandler;

#[derive(Debug, Clone, Data, Lens)]
pub struct AppData {
    items: Arc<Vec<String>>, // FIXME I must split GUI from Logic
    port_name: String, // FIXME data should be cheap to clone but lens can't access to Arc<String> ?
    baud_rate: String, // FIXME data should be cheap to clone but lens can't access to Arc<String> ?
    to_write: String,  // FIXME data should be cheap to clone but lens can't access to Arc<String> ?
    data_bits: DruidDataBits,
    flow_control: DruidFlowControl,
    parity: DruidParity,
    stop_bits: DruidStopBits,
    protocol: Protocol,
    sender: Arc<Sender<GuiMessage>>,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler {}
    }
}

impl Widget<AppData> for EventHandler {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            // FIXME mix READ and WRITE item is not clean currently
            Event::Command(cmd) if cmd.selector == WRITE_ITEM => {
                let items = Arc::make_mut(&mut data.items);
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
            }
            Event::Command(cmd) if cmd.selector == READ_ITEM => {
                let items = Arc::make_mut(&mut data.items);

                if items.is_empty() {
                    items.push("".to_string());
                }

                // Update last item until it grow to a certain size then add a new one
                let mut items_idx = items.len() - 1;
                let chunk_size: usize = 45;

                match data.protocol {
                    Protocol::Lines => {
                        let to_insert = format!(
                            "{} {}",
                            items[items_idx],
                            cmd.get_object::<String>().unwrap()
                        );

                        for line in to_insert.lines() {
                            items[items_idx].clear();
                            items[items_idx].insert_str(0, line);
                            items.push("".to_string());
                            items_idx += 1;
                        }
                    }
                    Protocol::Raw => {
                        let to_insert = format!(
                            "{} {}",
                            items[items_idx],
                            cmd.get_object::<String>()
                                .unwrap()
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

                        // FIXME poor solution to split String
                        let to_insert = to_insert
                            .as_bytes()
                            .chunks(chunk_size)
                            .map(|buf| String::from_utf8_lossy(&buf[..]))
                            .collect::<Vec<_>>();

                        for i in to_insert {
                            items[items_idx].clear();
                            items[items_idx] = i.to_string();
                            if i.len() == chunk_size {
                                items.push("".to_string());
                                items_idx += 1;
                            }
                        }
                    }
                }

                ctx.request_layout();
                ctx.request_paint();
            }
            Event::Command(cmd) if cmd.selector == OPEN_PORT => {
                // FIXME protocol should be update on click in its RadioGroup
                data.sender
                    .send(GuiMessage::UpdateProtocol(data.protocol))
                    .unwrap();

                data.sender
                    .send(GuiMessage::Open(OpenMessage {
                        port_name: data.port_name.clone(),
                        baud_rate: data.baud_rate.clone(),
                        data_bits: data.data_bits,
                        flow_control: data.flow_control,
                        parity: data.parity,
                        stop_bits: data.stop_bits,
                        protocol: data.protocol,
                    }))
                    .unwrap()
            }
            Event::Command(cmd) if cmd.selector == CLOSE_PORT => {
                data.sender.send(GuiMessage::Close).unwrap()
            }
            Event::Command(cmd) if cmd.selector == WRITE_PORT => {
                // FIXME conversion
                let bytes = data.to_write.as_bytes();
                data.sender
                    .send(GuiMessage::Write(bytes.to_owned()))
                    .unwrap()
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
    let window = WindowDesc::new(stool_ui::make_ui)
        .title(LocalizedString::new("Serial tool").with_placeholder("Stool"))
        .window_size((500., 850.)); // TODO check I think i should not need to do this by myself

    let launcher = AppLauncher::with_window(window);

    let event_sink = launcher.get_external_handle();

    let (sender, receiver) = channel::<GuiMessage>();

    thread::spawn(move || {
        // Create the runtime
        let mut async_rt = Runtime::new().unwrap();
        async_rt.block_on(async_serial::serial_loop(&event_sink, receiver));
    });

    launcher
        .launch(AppData {
            items: Arc::new(vec![]),
            port_name: "".to_string(),
            baud_rate: "115200".to_string(),
            to_write: "".to_string(),
            data_bits: DruidDataBits::Eight,
            flow_control: DruidFlowControl::None,
            parity: DruidParity::None,
            stop_bits: DruidStopBits::One,
            protocol: Protocol::Raw,
            sender: Arc::new(sender),
        })
        .expect("launch failed");
}
