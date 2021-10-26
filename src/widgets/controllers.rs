use std::sync::Arc;

use crate::data::AppData;
use crate::CLEAR_DATA;

use druid::{
    commands::{COPY, CUT, PASTE},
    Command, Target,
};
use druid::{widget::Controller, FileDialogOptions, FileSpec};
use druid::{Data, Menu, MenuItem};
use druid::{Env, Event, EventCtx, HotKey, KbKey, LocalizedString, SysMods, UpdateCtx, Widget};

use tokio_serial;

#[derive(Debug, Default)]
pub struct ContextMenuController;

impl<T, W: Widget<T>> Controller<T, W> for ContextMenuController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::MouseDown(ref mouse) if mouse.button.is_right() => {
                let mut pos = mouse.pos;
                pos.x += 150.; // Note: 150 is the size from the left menu bar
                ctx.show_context_menu(make_context_menu::<AppData>(), pos);
            }
            _ => child.event(ctx, event, data, env),
        }
    }
}

fn make_context_menu<T: Data>() -> Menu<T> {
    Menu::empty()
        .entry(
            MenuItem::new(LocalizedString::new("Clear"))
                .on_activate(|ctx, _data, _env| ctx.submit_command(CLEAR_DATA)),
        )
        .entry(
            MenuItem::new(LocalizedString::new("Export")).on_activate(|ctx, _data, _env| {
                let save_dialog_options = FileDialogOptions::new()
                    .allowed_types(vec![FileSpec::new("Text file", &["txt"])])
                    .default_type(FileSpec::new("Text file", &["txt"]))
                    .default_name(String::from("MyFile.txt"))
                    .name_label("Target")
                    .title("Choose a target for this lovely file")
                    .button_text("Export");

                ctx.submit_command(Command::new(
                    druid::commands::SHOW_SAVE_PANEL,
                    save_dialog_options,
                    Target::Auto,
                ))
            }),
        )
}

#[derive(Debug, Default)]
pub struct PortTextBoxController;

impl<W: Widget<AppData>> Controller<AppData, W> for PortTextBoxController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppData,
        env: &Env,
    ) {
        match event {
            Event::MouseDown(_) => {
                let mut available_ports = tokio_serial::available_ports()
                    .unwrap()
                    .iter()
                    .map(|pinfo| pinfo.port_name.clone())
                    .collect::<Vec<String>>();
                available_ports.sort();
                data.port_name = Arc::new(available_ports.join(" "));
                child.event(ctx, event, data, env);
            }
            Event::KeyUp(key_event) => match key_event {
                k_e if (HotKey::new(None, KbKey::Tab)).matches(k_e) => {
                    let mut available_ports = tokio_serial::available_ports()
                        .unwrap()
                        .iter()
                        .map(|pinfo| pinfo.port_name.clone())
                        .collect::<Vec<String>>();
                    available_ports.sort();
                    data.port_name = Arc::new(available_ports.join(" "));
                }
                _ => {
                    child.event(ctx, event, data, env);
                }
            },
            Event::KeyDown(key_event) => match key_event {
                k_e if (HotKey::new(SysMods::Cmd, "x")).matches(k_e) => {
                    ctx.submit_command(CUT);
                }
                k_e if (HotKey::new(SysMods::Cmd, "c")).matches(k_e) => {
                    ctx.submit_command(COPY);
                }
                k_e if (HotKey::new(SysMods::Cmd, "v")).matches(k_e) => {
                    ctx.submit_command(PASTE);
                }
                _ => {
                    child.event(ctx, event, data, env);
                }
            },
            other => {
                child.event(ctx, other, data, env);
            }
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

#[derive(Debug, Default)]
pub struct TextBoxController;

impl<W: Widget<AppData>> Controller<AppData, W> for TextBoxController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppData,
        env: &Env,
    ) {
        match event {
            Event::KeyDown(key_event) => match key_event {
                k_e if (HotKey::new(SysMods::Cmd, "x")).matches(k_e) => {
                    ctx.submit_command(CUT);
                }
                k_e if (HotKey::new(SysMods::Cmd, "c")).matches(k_e) => {
                    ctx.submit_command(COPY);
                }
                k_e if (HotKey::new(SysMods::Cmd, "v")).matches(k_e) => {
                    ctx.submit_command(PASTE);
                }
                _ => child.event(ctx, event, data, env),
            },
            //Event::Timer(_) => {} To remove the blink
            other => {
                child.event(ctx, other, data, env);
            }
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
