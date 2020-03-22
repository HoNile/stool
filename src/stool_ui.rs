use druid::widget::{
    Button, CrossAxisAlignment, Flex, Label, List, RadioGroup, Scroll, SizedBox, TextBox, WidgetExt,
};
use druid::{Color, Command, Data, LocalizedString, Widget};
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
        .with_child(SizedBox::empty().width(150.).height(40.))
        .with_flex_child(TextBox::new().expand_width().lens(AppData::to_write), 1.0)
        .with_spacer(6.)
        .with_child(
            Button::new(
                LocalizedString::new("Send"),
                |ctx, data: &mut AppData, _env| {
                    ctx.submit_command(Command::new(WRITE_PORT, data.clone()), None);
                },
            )
            .fix_width(110.0),
        )
        .with_child(SizedBox::empty().width(6.))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .background(Color::rgb8(0x1a, 0x1a, 0x1a));

    let control_panel = Flex::column()
        .with_child(
            Label::new(LocalizedString::new("Available ports:"))
                .fix_width(110.0)
                .padding((20., 20., 20., 0.)),
        )
        .with_spacer(3.)
        .with_child(Label::new(list_ports).fix_width(110.0))
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Port:")))
        .with_spacer(3.)
        .with_child(TextBox::new().fix_width(110.0).lens(AppData::port_name))
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Baudrate:")))
        .with_spacer(3.)
        .with_child(TextBox::new().fix_width(110.0).lens(AppData::baud_rate))
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Data bits:")))
        .with_spacer(3.)
        .with_child(
            Flex::column()
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
                )
                .cross_axis_alignment(CrossAxisAlignment::Start),
        )
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Flow control:")))
        .with_spacer(3.)
        .with_child(
            Flex::column()
                .with_child(
                    RadioGroup::new(vec![
                        (LocalizedString::new("None"), DruidFlowControl::None),
                        (LocalizedString::new("Hardware"), DruidFlowControl::Hardware),
                        (LocalizedString::new("Software"), DruidFlowControl::Software),
                    ])
                    .fix_width(110.0)
                    .border(Color::grey(0.6), 2.0)
                    .rounded(5.0)
                    .lens(AppData::flow_control),
                )
                .cross_axis_alignment(CrossAxisAlignment::Start),
        )
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Parity:")))
        .with_spacer(3.)
        .with_child(
            Flex::column()
                .with_child(
                    RadioGroup::new(vec![
                        (LocalizedString::new("None"), DruidParity::None),
                        (LocalizedString::new("Even"), DruidParity::Even),
                        (LocalizedString::new("Odd"), DruidParity::Odd),
                    ])
                    .fix_width(110.0)
                    .border(Color::grey(0.6), 2.0)
                    .rounded(5.0)
                    .lens(AppData::parity),
                )
                .cross_axis_alignment(CrossAxisAlignment::Start),
        )
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Stop bits:")))
        .with_spacer(3.)
        .with_child(
            Flex::column()
                .with_child(
                    RadioGroup::new(vec![
                        (LocalizedString::new("One"), DruidStopBits::One),
                        (LocalizedString::new("Two"), DruidStopBits::Two),
                    ])
                    .fix_width(110.0)
                    .border(Color::grey(0.6), 2.0)
                    .rounded(5.0)
                    .lens(AppData::stop_bits),
                )
                .cross_axis_alignment(CrossAxisAlignment::Start),
        )
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Protocol:")))
        .with_spacer(3.)
        .with_child(
            Flex::column()
                .with_child(
                    RadioGroup::new(vec![
                        (LocalizedString::new("Lines"), Protocol::Lines),
                        (LocalizedString::new("Raw"), Protocol::Raw),
                    ])
                    .fix_width(110.0)
                    .border(Color::grey(0.6), 2.0)
                    .rounded(5.0)
                    .lens(AppData::protocol),
                )
                .cross_axis_alignment(CrossAxisAlignment::Start),
        )
        .with_spacer(6.)
        .with_child(
            Button::new(
                LocalizedString::new("Open port"),
                |ctx, data: &mut AppData, _env| {
                    ctx.submit_command(Command::new(OPEN_PORT, data.clone()), None);
                },
            )
            .fix_width(110.0),
        )
        .with_spacer(6.)
        .with_child(
            Button::new(
                LocalizedString::new("Close port"),
                |ctx, data: &mut AppData, _env| {
                    ctx.submit_command(Command::new(CLOSE_PORT, data.clone()), None);
                },
            )
            .fix_width(110.0),
        )
        .with_flex_spacer(1.0)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .background(Color::rgb8(0x1a, 0x1a, 0x1a));

    Flex::column()
        .with_child(EventHandler::new().fix_width(0.0).fix_height(0.0))
        .with_flex_child(
            Flex::row().with_child(control_panel).with_flex_child(
                Scroll::new(
                    List::new(|| Label::new(|item: &String, _env: &_| item.to_string()))
                        .lens(AppData::items),
                )
                .expand(),
                1.0,
            ),
            1.0,
        )
        .with_child(write_panel)
}
