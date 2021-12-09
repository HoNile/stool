use crate::data::{AppData, OpenMessage, OutputTag, Protocol};
use crate::serial::{ByteDirection, IO_DATA, IO_ERROR};
use bytes::Bytes;
use druid::piet::TextStorage;
use druid::text::{Attribute, RichText};
use druid::{
    BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, Selector, Size, UpdateCtx,
};
use std::collections::VecDeque;
use std::{ops::Range, sync::Arc};

use druid::Widget;

pub const OPEN_PORT: Selector = Selector::new("event.open-port");
pub const CLOSE_PORT: Selector = Selector::new("event.close-port");
pub const WRITE_PORT: Selector = Selector::new("event.write-port");
pub const CLEAR_DATA: Selector = Selector::new("event.clear-data");

const MAX_VIEW_SIZE: usize = 1024 * 180;

#[derive(Debug, Clone, PartialEq)]
pub enum GuiMessage {
    Open(OpenMessage),
    Close,
    Write(Bytes),
}

pub fn get_tag_color(tag: OutputTag) -> Color {
    match tag {
        OutputTag::TextIn => Color::rgb8(128, 0, 255),
        OutputTag::TextOut => Color::rgb8(50, 190, 220),
        OutputTag::RawIn => Color::rgb8(25, 155, 35),
        OutputTag::RawOut => Color::rgb8(240, 160, 25),
    }
}

pub fn display_raw(
    io_data: &(ByteDirection, Bytes),
    output: &mut RichText,
    mut output_attr: &mut Arc<VecDeque<(Range<usize>, OutputTag)>>,
) {
    let curr_output = output.as_str();
    let before_insert_len = curr_output.len();

    match io_data.0 {
        ByteDirection::Out => {
            let to_print = format!(
                "{}\n",
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

            if curr_output.is_empty() || curr_output.chars().rev().take(1).next() == Some('\n') {
                *output = RichText::new(format!("{}{}", curr_output, to_print).into());
            } else {
                *output = RichText::new(format!("{}\n{}", curr_output, to_print).into());
            }

            Arc::make_mut(&mut output_attr)
                .push_back((before_insert_len..output.len(), OutputTag::RawOut));
        }
        ByteDirection::In => {
            let new_data = hex::encode_upper(&io_data.1);

            if output_attr.is_empty() {
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
                *output = RichText::new(format!("{}{}", curr_output, to_insert).into());
            } else {
                match &output_attr[output_attr.len() - 1].1 {
                    OutputTag::RawIn => {
                        let output_last_line: Option<&str> =
                            output.as_str().lines().rev().take(1).next();

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
                                if new_output.is_empty() {
                                    new_output = line.to_string();
                                } else {
                                    new_output = format!("{}\n{}", new_output, line);
                                }
                            }
                            if new_output.is_empty() {
                                new_output = to_insert;
                            } else {
                                new_output = format!("{}\n{}", new_output, to_insert);
                            }
                            *output = RichText::new(new_output.into());
                        }
                    }
                    OutputTag::TextIn => {
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
                        *output = RichText::new(format!("{}\n{}", curr_output, to_insert).into());
                    }
                    _ => {
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
                        *output = RichText::new(format!("{}{}", curr_output, to_insert).into());
                    }
                }
            };

            Arc::make_mut(&mut output_attr)
                .push_back((before_insert_len..output.len(), OutputTag::RawIn));
        }
    }
}

pub fn display_text(
    io_data: &(ByteDirection, Bytes),
    output: &mut RichText,
    mut output_attr: &mut Arc<VecDeque<(Range<usize>, OutputTag)>>,
) {
    let curr_output = output.as_str();
    let before_insert_len = curr_output.len();

    match io_data.0 {
        ByteDirection::Out => {
            let to_print = format!("{}\n", String::from_utf8_lossy(&io_data.1));

            if curr_output.is_empty() || curr_output.chars().rev().take(1).next() == Some('\n') {
                *output = RichText::new(format!("{}{}", curr_output, to_print).into());
            } else {
                *output = RichText::new(format!("{}\n{}", curr_output, to_print).into());
            }

            Arc::make_mut(&mut output_attr)
                .push_back((before_insert_len..output.len(), OutputTag::TextOut));
        }
        ByteDirection::In => {
            if output_attr.is_empty() {
                let to_print =
                    format!("{}{}", output.as_str(), String::from_utf8_lossy(&io_data.1));
                *output = RichText::new(to_print.into());
            } else {
                match &output_attr[output_attr.len() - 1].1 {
                    OutputTag::TextIn => {
                        let to_print =
                            format!("{}{}", output.as_str(), String::from_utf8_lossy(&io_data.1));
                        *output = RichText::new(to_print.into());
                    }
                    OutputTag::RawIn => {
                        let to_print = format!(
                            "{}\n{}",
                            output.as_str(),
                            String::from_utf8_lossy(&io_data.1)
                        );
                        *output = RichText::new(to_print.into());
                    }
                    _ => {
                        let to_print =
                            format!("{}{}", output.as_str(), String::from_utf8_lossy(&io_data.1));
                        *output = RichText::new(to_print.into());
                    }
                }
            };

            Arc::make_mut(&mut output_attr)
                .push_back((before_insert_len..output.len(), OutputTag::TextIn));
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
                    Protocol::Raw => display_raw(io_data, &mut data.output, &mut data.output_attr),
                    Protocol::Text => {
                        display_text(io_data, &mut data.output, &mut data.output_attr)
                    }
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

                    let out_attr = Arc::make_mut(&mut data.output_attr);

                    for attr in out_attr.iter_mut() {
                        attr.0.start = attr.0.start.saturating_sub(output_len - MAX_VIEW_SIZE);
                        attr.0.end = attr.0.end.saturating_sub(output_len - MAX_VIEW_SIZE);
                    }

                    loop {
                        if out_attr[0].0.start > 0 {
                            out_attr.pop_front();
                        } else {
                            break;
                        }
                    }
                }

                for attr in data.output_attr.iter() {
                    data.output.add_attribute(
                        attr.0.clone(),
                        Attribute::text_color(get_tag_color(attr.1.clone())),
                    );
                }
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

                data.status = format!(
                    "{}, {}, {}, {}, {}",
                    (*data.port_name).clone(),
                    data.baud_rate,
                    data.flow_control,
                    data.parity,
                    data.stop_bits
                );
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
                            .unbounded_send(GuiMessage::Write(bytes.into()))
                            .unwrap();
                    } else {
                        data.status = "Incorrect data doesn't respect protocol format".to_string();
                    }
                }
                Protocol::Text => {
                    let bytes = data.to_write.as_bytes().to_owned();
                    data.sender
                        .unbounded_send(GuiMessage::Write(bytes.into()))
                        .unwrap();
                }
            },
            Event::Command(cmd) if cmd.is(IO_ERROR) => {
                let error_msg = cmd.get_unchecked(IO_ERROR);
                data.status = error_msg.to_string();
            }
            Event::Command(cmd) if cmd.is(CLEAR_DATA) => {
                data.output = RichText::new("".into());
                Arc::make_mut(&mut data.output_attr).clear();
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
