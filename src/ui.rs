use crate::event::{EventHandler, CLOSE_PORT, OPEN_PORT, WRITE_PORT};
use crate::widgets::NumericFormatter;
use crate::{
    data::{
        AppData, DruidDataBits, DruidFlowControl, DruidParity, DruidStopBits, PortNameLens,
        Protocol, ToWriteLens,
    },
    widgets::{ContextMenuController, PortTextBoxController, TextBoxController},
};

use druid::widget::{
    Button, CrossAxisAlignment, Flex, Label, LineBreaking, RadioGroup, RawLabel, Scroll, SizedBox,
    TextBox, WidgetExt,
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
            TextBox::new()
                .fix_width(110.0)
                .lens(PortNameLens)
                .controller(PortTextBoxController::default()),
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
                (LocalizedString::new("Text"), Protocol::Text),
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
        .with_flex_spacer(1.0)
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
