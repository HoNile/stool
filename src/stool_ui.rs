use druid::widget::{
    Button, CrossAxisAlignment, Flex, Label, List, RadioGroup, Scroll, SizedBox, TextBox, WidgetExt,
};
use druid::{Color, Command, Data, Widget};
use serialport;
use std::vec::Vec;

use crate::{AppData, EventHandler, CLOSE_PORT, OPEN_PORT, WRITE_PORT};

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum Protocol {
    Lines,
    Raw,
}

// FIXME should probably not be done like this
#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum DruidDataBits {
    Eight,
    Seven,
    Six,
    Five,
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum DruidFlowControl {
    Hardware,
    Software,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum DruidParity {
    Even,
    Odd,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Data)]
pub enum DruidStopBits {
    One,
    Two,
}
// END FIXME

#[derive(Debug, Clone, PartialEq, Data)]
pub struct OpenMessage {
    pub port_name: String,
    pub baud_rate: String,
    pub data_bits: DruidDataBits,
    pub flow_control: DruidFlowControl,
    pub parity: DruidParity,
    pub stop_bits: DruidStopBits,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuiMessage {
    Open(OpenMessage),
    Close,
    UpdateProtocol(Protocol),
    Write(Vec<u8>),
    Shutdown,
}

pub fn make_ui() -> impl Widget<AppData> {
    let list_ports: String = serialport::available_ports()
        .unwrap()
        .iter()
        .map(|pinfo| pinfo.port_name.clone())
        .collect::<Vec<String>>()
        .join(" ");

    let write_panel = Flex::row()
        .with_child(SizedBox::empty().width(150.).height(40.), 0.0)
        .with_spacer(3.)
        .with_child(TextBox::new().lens(AppData::to_write), 1.0)
        .with_spacer(6.)
        .with_child(
            Button::new("Send", |ctx, data: &mut AppData, _env| {
                ctx.submit_command(Command::new(WRITE_PORT, data.clone()), None);
            })
            .fix_width(110.0),
            0.0,
        )
        .with_child(SizedBox::empty().width(6.), 0.0)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .background(Color::rgb8(0x1a, 0x1a, 0x1a));

    let control_panel = Flex::column()
        .with_child(
            Flex::column()
                .with_child(
                    Label::new("Available ports:")
                        .fix_width(110.0)
                        .padding((20., 20., 20., 0.)),
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
            Flex::column()
                .with_child(Label::new("Protocol:"), 0.0)
                .with_spacer(3.)
                .with_child(
                    RadioGroup::new(vec![("Lines", Protocol::Lines), ("Raw", Protocol::Raw)])
                        .fix_width(110.0)
                        .border(Color::grey(0.6), 2.0)
                        .rounded(5.0)
                        .lens(AppData::protocol),
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
        .with_spacer(6.)
        .with_child(
            Button::new("Close port", |ctx, data: &mut AppData, _env| {
                ctx.submit_command(Command::new(CLOSE_PORT, data.clone()), None);
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
                    List::new(|| Label::new(|item: &String, _env: &_| format!("{}", item)))
                        .lens(AppData::items),
                )
                .expand(),
                1.0,
            ),
            1.0,
        )
        .with_child(write_panel, 0.0)
}
