mod async_serial;

use async_serial::ADD_ITEM;
use druid::widget::{
    Button, CrossAxisAlignment, Flex, Label, List, RadioGroup, Scroll, TextBox, WidgetExt, SizedBox
};
use druid::{
    AppLauncher, BoxConstraints, Color, Command, Data, Env, Event, EventCtx, LayoutCtx, Lens,
    LifeCycle, LifeCycleCtx, LocalizedString, PaintCtx, Selector, Size, UpdateCtx, Widget,
    WindowDesc,
};
use serialport;
use std::{
    sync::{
        mpsc::{channel, Sender},
        Arc,
    },
    thread,
};
use tokio::runtime::Runtime;
use tokio_serial::{DataBits, FlowControl, Parity, SerialPortSettings, StopBits};

const OPEN_PORT: Selector = Selector::new("event.open-port");

// FIXME should probably not be done like this
#[derive(Debug, Clone, Copy, PartialEq, Data)]
enum DruidDataBits {
    Eight,
    Seven,
    Six,
    Five,
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
enum DruidFlowControl {
    Hardware,
    Software,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
enum DruidParity {
    Even,
    Odd,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
enum DruidStopBits {
    One,
    Two,
}
// END FIXME

struct EventHandler;

type ItemData = String;

#[derive(Debug, Clone, Data, Lens)]
struct AppData {
    items: Arc<Vec<ItemData>>,
    port_name: String,
    baud_rate: String,
    data_bits: DruidDataBits,
    flow_control: DruidFlowControl,
    parity: DruidParity,
    stop_bits: DruidStopBits,
    sender: Arc<
        Sender<(
            String,
            String,
            DruidDataBits,
            DruidFlowControl,
            DruidParity,
            DruidStopBits,
        )>,
    >,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler {}
    }
}

impl Widget<AppData> for EventHandler {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::Command(cmd) if cmd.selector == ADD_ITEM => {
                let items = Arc::make_mut(&mut data.items);

                if items.len() == 0 {
                    items.push("".to_string());
                }

                // Update last item until it grow to a certain size then add a new one
                let items_idx = items.len() - 1;
                let str_len = items[items_idx].len();
                items[items_idx].insert_str(str_len, cmd.get_object::<ItemData>().unwrap());
                let str_len = items[items_idx].len();
                items[items_idx].insert_str(str_len, " ");

                if items[items_idx].len() >= 45 {
                    items.push("".to_string());
                }

                ctx.request_layout();
                ctx.request_paint();
            }
            Event::Command(cmd) if cmd.selector == OPEN_PORT => data
                .sender
                .send((
                    data.port_name.clone(),
                    data.baud_rate.clone(),
                    data.data_bits,
                    data.flow_control,
                    data.parity,
                    data.stop_bits,
                ))
                .unwrap(),
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
        .title(LocalizedString::new("Serial tool").with_placeholder("Stool"));

    let launcher = AppLauncher::with_window(window);

    let event_sink = launcher.get_external_handle();

    let (sender, receiver) = channel::<(
        String,
        String,
        DruidDataBits,
        DruidFlowControl,
        DruidParity,
        DruidStopBits,
    )>();

    thread::spawn(move || {
        let mut settings = SerialPortSettings::default();
        // Create the runtime
        let mut async_rt = Runtime::new().unwrap();

        if let Ok(gui_settings) = receiver.recv() {
            let name = gui_settings.0.as_str();

            settings.baud_rate = gui_settings.1.parse::<u32>().unwrap();

            settings.data_bits = match gui_settings.2 {
                DruidDataBits::Eight => DataBits::Eight,
                DruidDataBits::Seven => DataBits::Seven,
                DruidDataBits::Six => DataBits::Six,
                DruidDataBits::Five => DataBits::Five,
            };

            settings.flow_control = match gui_settings.3 {
                DruidFlowControl::Hardware => FlowControl::Hardware,
                DruidFlowControl::Software => FlowControl::Software,
                DruidFlowControl::None => FlowControl::None,
            };

            settings.parity = match gui_settings.4 {
                DruidParity::Even => Parity::Even,
                DruidParity::Odd => Parity::Odd,
                DruidParity::None => Parity::None,
            };

            settings.stop_bits = match gui_settings.5 {
                DruidStopBits::One => StopBits::One,
                DruidStopBits::Two => StopBits::Two,
            };

            async_rt.block_on(async_serial::serial_loop(&event_sink, &settings, name));
        }
    });

    launcher
        .use_simple_logger()
        .launch(AppData {
            items: Arc::new(vec![]),
            port_name: "".to_string(),
            baud_rate: "115200".to_string(),
            data_bits: DruidDataBits::Eight,
            flow_control: DruidFlowControl::None,
            parity: DruidParity::None,
            stop_bits: DruidStopBits::One,
            sender: Arc::new(sender),
        })
        .expect("launch failed");
}

fn make_ui() -> impl Widget<AppData> {
    let list_ports: String = serialport::available_ports()
        .unwrap()
        .iter()
        .map(|pinfo| pinfo.port_name.clone())
        .collect::<Vec<String>>()
        .join(" ");

    let control_panel = Flex::column()
        .with_child(
            Flex::column()
                .with_child(
                    Label::new("Available ports:")
                        .fix_width(110.0)
                        .padding(20.0),
                    0.0,
                )
                .with_spacer(3.)
                .with_child(Label::new(list_ports).fix_width(110.0), 0.0),
            0.0,
        )
        .with_spacer(6.)
        .with_child(
            Flex::column()
                .with_child(Label::new("Port:"), 0.0)
                .with_spacer(3.)
                .with_child(
                    TextBox::new().fix_width(110.0).lens(AppData::port_name),
                    0.0,
                ),
            0.0,
        )
        .with_spacer(6.)
        .with_child(
            Flex::column()
                .with_child(Label::new("Baudrate:"), 0.0)
                .with_spacer(3.)
                .with_child(
                    TextBox::new().fix_width(110.0).lens(AppData::baud_rate),
                    0.0,
                ),
            0.0,
        )
        .with_spacer(6.)
        .with_child(
            Flex::column()
                .with_child(Label::new("Data bits:"), 0.0)
                .with_spacer(3.)
                .with_child(
                    RadioGroup::new(vec![
                        ("8", DruidDataBits::Eight),
                        ("7", DruidDataBits::Seven),
                        ("6", DruidDataBits::Six),
                        ("5", DruidDataBits::Five),
                    ])
                    .fix_width(110.0)
                    .border(Color::grey(0.6), 2.0)
                    .rounded(5.0)
                    .lens(AppData::data_bits),
                    0.0,
                ),
            0.0,
        )
        .with_spacer(6.)
        .with_child(
            Flex::column()
                .with_child(Label::new("Flow control:"), 0.0)
                .with_spacer(3.)
                .with_child(
                    RadioGroup::new(vec![
                        ("None", DruidFlowControl::None),
                        ("Hardware", DruidFlowControl::Hardware),
                        ("Software", DruidFlowControl::Software),
                    ])
                    .fix_width(110.0)
                    .border(Color::grey(0.6), 2.0)
                    .rounded(5.0)
                    .lens(AppData::flow_control),
                    0.0,
                ),
            0.0,
        )
        .with_spacer(6.)
        .with_child(
            Flex::column()
                .with_child(Label::new("Parity:"), 0.0)
                .with_spacer(3.)
                .with_child(
                    RadioGroup::new(vec![
                        ("None", DruidParity::None),
                        ("Even", DruidParity::Even),
                        ("Odd", DruidParity::Odd),
                    ])
                    .fix_width(110.0)
                    .border(Color::grey(0.6), 2.0)
                    .rounded(5.0)
                    .lens(AppData::parity),
                    0.0,
                ),
            0.0,
        )
        .with_spacer(6.)
        .with_child(
            Flex::column()
                .with_child(Label::new("Stop bits:"), 0.0)
                .with_spacer(3.)
                .with_child(
                    RadioGroup::new(vec![
                        ("One", DruidStopBits::One),
                        ("Two", DruidStopBits::Two),
                    ])
                    .fix_width(110.0)
                    .border(Color::grey(0.6), 2.0)
                    .rounded(5.0)
                    .lens(AppData::stop_bits),
                    0.0,
                ),
            0.0,
        )
        .with_spacer(6.)
        .with_child(
            Button::new("Open port", |ctx, data: &mut AppData, _env| {
                ctx.submit_command(Command::new(OPEN_PORT, data.clone()), None);
            })
            .fix_width(110.0),
            0.0,
        )
        .with_child(SizedBox::empty(), 1.0)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .background(Color::rgb8(0x1a, 0x1a, 0x1a));

    Flex::column()
        .with_child(EventHandler::new().fix_width(0.0).fix_height(0.0), 0.0)
        .with_child(
            Flex::row().with_child(control_panel, 0.0).with_child(
                Scroll::new(
                    List::new(|| {
                        Button::new(
                            |item: &ItemData, _env: &_| format!("{}", item),
                            |_ctx, _data, _env| {},
                        )
                    })
                    .lens(AppData::items),
                )
                .vertical(),
                1.0,
            ),
            1.0,
        )
}
