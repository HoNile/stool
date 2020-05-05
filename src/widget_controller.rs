//! Controller widgets

use druid::widget::Controller;
use druid::{Command, Env, Event, EventCtx, UpdateCtx, Widget};

use crate::AppData;

/// A widget that wraps all root widgets
#[derive(Debug, Default)]
pub struct RootWindowController;

impl<W: Widget<AppData>> Controller<AppData, W> for RootWindowController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppData,
        env: &Env,
    ) {
        match event {
            Event::WindowSize(size) => data.line_size = (size.width / 8.) as usize - 15,
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
