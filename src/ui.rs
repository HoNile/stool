use crate::widgets::{Dropdown, ListSelect, NumericFormatter, DROP};
use crate::{
    data::{
        AppData, DruidDataBits, DruidFlowControl, DruidParity, DruidStopBits, Protocol, ToWriteLens,
    },
    widgets::{ContextMenuController, TextBoxController},
};
use crate::{EventHandler, CLOSE_PORT, OPEN_PORT, WRITE_PORT};

use druid::{
    widget::{
        Button, CrossAxisAlignment, Flex, Label, LineBreaking, RawLabel, Scroll, SizedBox, TextBox,
        WidgetExt,
    },
    Env, EventCtx,
};
use druid::{Color, FontDescriptor, FontFamily, LocalizedString, Widget};

pub fn make_ui() -> impl Widget<AppData> {
    let write_panel = Flex::column()
        .with_child(SizedBox::empty().height(8.))
        .with_child(
            Flex::row()
                .with_flex_child(
                    TextBox::multiline()
                        .expand_width()
                        .lens(ToWriteLens)
                        .controller(TextBoxController::default()),
                    1.0,
                )
                .with_spacer(6.)
                .with_child(
                    Button::new(LocalizedString::new("Send"))
                        .on_click(|ctx, _data, _env| {
                            ctx.submit_command(WRITE_PORT);
                        })
                        .fix_width(110.0),
                )
                .with_child(SizedBox::empty().width(6.))
                .cross_axis_alignment(CrossAxisAlignment::Center),
        )
        .with_child(SizedBox::empty().height(8.))
        .background(Color::rgb8(0x1a, 0x1a, 0x1a));

    let control_panel = Flex::column()
        .with_spacer(5.)
        .with_child(Label::new(LocalizedString::new("Port:")))
        .with_spacer(3.)
        .with_child(
            Dropdown::new(
                Flex::row()
                    .with_flex_spacer(1.)
                    .with_child(
                        Label::new(|p: &String, _: &Env| format!("{}", p))
                            .background(Color::rgb8(58, 58, 58))
                            .fix_width(80.0),
                    )
                    .with_child(
                        Button::new("v")
                            .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROP)),
                    )
                    .with_flex_spacer(1.),
                |_, _| {
                    let available_ports = serialport::available_ports()
                        .unwrap()
                        .iter()
                        .map(|pinfo| (pinfo.port_name.clone(), pinfo.port_name.clone()))
                        .collect::<Vec<(String, String)>>();
                    Scroll::new(ListSelect::new(available_ports)).vertical()
                },
            )
            .align_left()
            .lens(AppData::port_name),
        )
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Baudrate:")))
        .with_spacer(3.)
        .with_child(
            TextBox::new()
                .with_formatter(NumericFormatter)
                .fix_width(110.0)
                .lens(AppData::baud_rate)
                .controller(TextBoxController::default()),
        )
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Data bits:")))
        .with_spacer(3.)
        .with_child(
            Dropdown::new(
                Flex::row()
                    .with_flex_spacer(1.)
                    .with_child(
                        Label::new(|db: &DruidDataBits, _: &Env| format!("{}", db))
                            .background(Color::rgb8(58, 58, 58))
                            .fix_width(80.0),
                    )
                    .with_child(
                        Button::new("v")
                            .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROP)),
                    )
                    .with_flex_spacer(1.),
                |_, _| {
                    Scroll::new(ListSelect::new(vec![
                        ("8", DruidDataBits::Eight),
                        ("7", DruidDataBits::Seven),
                        ("6", DruidDataBits::Six),
                        ("5", DruidDataBits::Five),
                    ]))
                    .vertical()
                },
            )
            .align_left()
            .lens(AppData::data_bits),
        )
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Flow control:")))
        .with_spacer(3.)
        .with_child(
            Dropdown::new(
                Flex::row()
                    .with_flex_spacer(1.)
                    .with_child(
                        Label::new(|fc: &DruidFlowControl, _: &Env| format!("{}", fc))
                            .background(Color::rgb8(58, 58, 58))
                            .fix_width(80.0),
                    )
                    .with_child(
                        Button::new("v")
                            .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROP)),
                    )
                    .with_flex_spacer(1.),
                |_, _| {
                    Scroll::new(ListSelect::new(vec![
                        ("None", DruidFlowControl::None),
                        ("Hardware", DruidFlowControl::Hardware),
                        ("Software", DruidFlowControl::Software),
                    ]))
                    .vertical()
                },
            )
            .align_left()
            .lens(AppData::flow_control),
        )
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Parity:")))
        .with_spacer(3.)
        .with_child(
            Dropdown::new(
                Flex::row()
                    .with_flex_spacer(1.)
                    .with_child(
                        Label::new(|p: &DruidParity, _: &Env| format!("{}", p))
                            .background(Color::rgb8(58, 58, 58))
                            .fix_width(80.0),
                    )
                    .with_child(
                        Button::new("v")
                            .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROP)),
                    )
                    .with_flex_spacer(1.),
                |_, _| {
                    Scroll::new(ListSelect::new(vec![
                        ("None", DruidParity::None),
                        ("Even", DruidParity::Even),
                        ("Odd", DruidParity::Odd),
                    ]))
                    .vertical()
                },
            )
            .align_left()
            .lens(AppData::parity),
        )
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Stop bits:")))
        .with_spacer(3.)
        .with_child(
            Dropdown::new(
                Flex::row()
                    .with_flex_spacer(1.)
                    .with_child(
                        Label::new(|sb: &DruidStopBits, _: &Env| format!("{}", sb))
                            .background(Color::rgb8(58, 58, 58))
                            .fix_width(80.0),
                    )
                    .with_child(
                        Button::new("v")
                            .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROP)),
                    )
                    .with_flex_spacer(1.),
                |_, _| {
                    Scroll::new(ListSelect::new(vec![
                        ("One", DruidStopBits::One),
                        ("Two", DruidStopBits::Two),
                    ]))
                    .vertical()
                },
            )
            .align_left()
            .lens(AppData::stop_bits),
        )
        .with_spacer(6.)
        .with_child(Label::new(LocalizedString::new("Protocol:")))
        .with_spacer(3.)
        .with_child(
            Dropdown::new(
                Flex::row()
                    .with_flex_spacer(1.)
                    .with_child(
                        Label::new(|p: &Protocol, _: &Env| format!("{:?}", p))
                            .background(Color::rgb8(58, 58, 58))
                            .fix_width(80.0),
                    )
                    .with_child(
                        Button::new("v")
                            .on_click(|ctx: &mut EventCtx, _, _| ctx.submit_notification(DROP)),
                    )
                    .with_flex_spacer(1.),
                |_, _| {
                    Scroll::new(ListSelect::new(vec![
                        ("Text", Protocol::Text),
                        ("Raw", Protocol::Raw),
                    ]))
                    .vertical()
                },
            )
            .align_left()
            .lens(AppData::protocol),
        )
        .with_flex_spacer(1.0)
        .with_child(
            Button::new(LocalizedString::new("Open port"))
                .on_click(|ctx, _data, _env| {
                    ctx.submit_command(OPEN_PORT);
                })
                .fix_width(110.0),
        )
        .with_spacer(6.)
        .with_child(
            Button::new(LocalizedString::new("Close port"))
                .on_click(|ctx, _data, _env| {
                    ctx.submit_command(CLOSE_PORT);
                })
                .fix_width(110.0),
        )
        .with_spacer(7.0)
        .background(Color::rgb8(0x1a, 0x1a, 0x1a))
        .fix_width(150.0);

    Flex::column()
        .with_child(EventHandler::new().fix_width(0.0).fix_height(0.0))
        .with_flex_child(
            Flex::row().with_child(control_panel).with_flex_child(
                Flex::column()
                    .with_flex_child(
                        Scroll::new(
                            RawLabel::new()
                                .with_font(
                                    FontDescriptor::new(FontFamily::MONOSPACE).with_size(18.),
                                )
                                .with_line_break_mode(LineBreaking::WordWrap)
                                .lens(AppData::output)
                                .expand_width(),
                        )
                        .vertical()
                        .expand()
                        .controller(ContextMenuController::default()),
                        1.0,
                    )
                    .with_child(write_panel),
                1.0,
            ),
            1.0,
        )
        .with_child(
            Label::new(|item: &String, _env: &_| item.to_string())
                .with_text_size(14.0)
                .fix_height(18.0)
                .padding((0., 0., 0., 2.))
                .lens(AppData::status),
        )
}
