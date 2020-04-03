use crate::data::{
    AppData, BaudRateLens, DruidDataBits, DruidFlowControl, DruidParity, DruidStopBits,
    PortNameLens, Protocol, ToWriteLens,
};
use crate::{EventHandler, CLOSE_PORT, OPEN_PORT, WRITE_PORT};
use druid::widget::{
    Button, CrossAxisAlignment, Flex, Label, List, RadioGroup, Scroll, SizedBox, TextBox, WidgetExt,
};
use druid::{Color, Command, LocalizedString, Widget};
use serialport;

pub fn make_ui() -> impl Widget<AppData> {
    let list_ports: String = serialport::available_ports()
        .unwrap()
        .iter()
        .map(|pinfo| pinfo.port_name.clone())
        .collect::<Vec<String>>()
        .join(" ");

    let write_panel = Flex::row()
        .with_child(SizedBox::empty().width(150.).height(40.))
        .with_flex_child(TextBox::new().expand_width().lens(ToWriteLens), 1.0)
        .with_spacer(6.)
        .with_child(
            Button::new(LocalizedString::new("Send"))
                .on_click(|ctx, data: &mut AppData, _env| {
                    ctx.submit_command(Command::new(WRITE_PORT, data.clone()), None);
                })
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
        .with_child(TextBox::new().fix_width(110.0).lens(PortNameLens))
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Baudrate:")))
        .with_spacer(3.)
        .with_child(TextBox::new().fix_width(110.0).lens(BaudRateLens))
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Data bits:")))
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
        )
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Flow control:")))
        .with_spacer(3.)
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
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Parity:")))
        .with_spacer(3.)
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
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Stop bits:")))
        .with_spacer(3.)
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
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Protocol:")))
        .with_spacer(3.)
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
        .with_spacer(6.)
        .with_child(
            Button::new(LocalizedString::new("Open port"))
                .on_click(|ctx, data: &mut AppData, _env| {
                    ctx.submit_command(Command::new(OPEN_PORT, data.clone()), None);
                })
                .fix_width(110.0),
        )
        .with_spacer(6.)
        .with_child(
            Button::new(LocalizedString::new("Close port"))
                .on_click(|ctx, data: &mut AppData, _env| {
                    ctx.submit_command(Command::new(CLOSE_PORT, data.clone()), None);
                })
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
                        .lens(AppData::visual_items),
                )
                .expand(),
                1.0,
            ),
            1.0,
        )
        .with_child(write_panel)
}
