use crate::data::AppData;
use druid::Env;
use druid::{commands, piet::TextStorage, AppDelegate, Command, DelegateCtx, Handled, Target};

pub struct Delegate;

impl AppDelegate<AppData> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppData,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::SAVE_FILE_AS) {
            if let Err(e) = std::fs::write(file_info.path(), data.output.as_str()) {
                println!("Error writing file: {}", e);
            }
            return Handled::Yes;
        }
        Handled::No
    }
}
