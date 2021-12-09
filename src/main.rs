#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod data;
mod delegate;
mod event;
mod serial;
mod ui;
mod widgets;

use crate::data::{AppData, DruidDataBits, DruidFlowControl, DruidParity, DruidStopBits};
use crate::ui::make_ui;
use data::Protocol;
use delegate::Delegate;
use druid::text::RichText;
use druid::{AppLauncher, LocalizedString, WindowDesc};
use event::GuiMessage;
use futures::channel::mpsc;
use std::collections::VecDeque;
use std::{sync::Arc, thread};
use tokio::runtime::Builder;

fn main() {
    let window = WindowDesc::new(make_ui())
        .title(LocalizedString::new("Serial tool").with_placeholder("Stool"))
        .with_min_size((164., 775.))
        .window_size((500., 775.));

    let launcher = AppLauncher::with_window(window);

    let event_sink = launcher.get_external_handle();

    let (sender, receiver) = mpsc::unbounded::<GuiMessage>();

    let rt_thread = thread::spawn(move || {
        // Create the runtime
        let async_rt = Builder::new_current_thread()
            .enable_io()
            .build()
            .expect("runtime failed");
        let _ = async_rt.block_on(serial::serial_loop(event_sink, receiver));
    });

    launcher
        .delegate(Delegate)
        .launch(AppData {
            output: RichText::new("".into()),
            output_attr: Arc::new(VecDeque::new()),
            port_name: Arc::new("".to_string()),
            baud_rate: 115_200,
            to_write: Arc::new("".to_string()),
            data_bits: DruidDataBits::Eight,
            flow_control: DruidFlowControl::None,
            parity: DruidParity::None,
            stop_bits: DruidStopBits::One,
            protocol: Protocol::Raw,
            sender: Arc::new(sender),
            status: "".to_string(),
        })
        .expect("launch failed");

    let _ = rt_thread.join();
}
