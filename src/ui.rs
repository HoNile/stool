use crate::data::{
    AppData, BaudRateLens, DruidDataBits, DruidFlowControl, DruidParity, DruidStopBits,
    PortNameLens, Protocol, ToWriteLens,
};
use crate::{EventHandler, CLOSE_PORT, OPEN_PORT, WRITE_PORT};
use druid::widget::{
    Button, CrossAxisAlignment, Flex, Label, LineBreaking, RadioGroup, RawLabel, Scroll, SizedBox,
    TextBox, WidgetExt,
};
use druid::{Color, FontDescriptor, FontFamily, LocalizedString, Widget};
use serialport;

use druid::commands::{COPY, CUT, PASTE};
use druid::keyboard_types::KeyState::Down;
use druid::widget::Controller;
use druid::Code::{KeyC, KeyV, KeyX};
use druid::{Env, Event, EventCtx, Modifiers, UpdateCtx};

#[derive(Debug, Default)]
pub struct ToWriteController;

impl<W: Widget<AppData>> Controller<AppData, W> for ToWriteController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppData,
        env: &Env,
    ) {
        match event {
            // FIXME probably not correct in all case
            Event::KeyDown(key_event) => {
                if key_event.state == Down
                    && key_event.code == KeyX
                    && key_event.mods & Modifiers::CONTROL == Modifiers::CONTROL
                {
                    ctx.submit_command(CUT);
                } else if key_event.state == Down
                    && key_event.code == KeyC
                    && key_event.mods & Modifiers::CONTROL == Modifiers::CONTROL
                {
                    ctx.submit_command(COPY);
                } else if key_event.state == Down
                    && key_event.code == KeyV
                    && key_event.mods & Modifiers::CONTROL == Modifiers::CONTROL
                {
                    ctx.submit_command(PASTE);
                } else {
                    child.event(ctx, event, data, env);
                }
            }
            other => child.event(ctx, other, data, env),
        }
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &AppData,
        data: &AppData,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env);
    }
}

pub fn make_ui() -> impl Widget<AppData> {
    let list_ports: String = serialport::available_ports()
        .unwrap()
        .iter()
        .map(|pinfo| pinfo.port_name.clone())
        .collect::<Vec<String>>()
        .join(" ");

    let write_panel = Flex::row()
        .with_flex_child(
            TextBox::new()
                .expand_width()
                .lens(ToWriteLens)
                .controller(ToWriteController::default()),
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
        .with_child(SizedBox::empty().width(6.).height(40.))
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .background(Color::rgb8(0x1a, 0x1a, 0x1a));

    let control_panel = Flex::column()
        .with_child(
            Label::new(LocalizedString::new("Available ports:"))
                .fix_width(110.0)
                .padding((20., 5., 20., 0.)),
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
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .background(Color::rgb8(0x1a, 0x1a, 0x1a));

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
                        .expand(),
                        1.0,
                    )
                    .with_child(write_panel),
                1.0,
            ),
            1.0,
        )
        .with_child(
            Label::new(|item: &String, _env: &_| item.to_string())
                .fix_height(17.0)
                .lens(AppData::status),
        )
}
