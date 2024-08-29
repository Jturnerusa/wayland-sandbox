// Copyright (C) 2024 John Turner

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{
    error::Error,
    fs::File,
    os::{
        fd::{AsFd, AsRawFd, FromRawFd},
        unix::ffi::OsStrExt,
    },
    path::PathBuf,
};

use clap::Parser;
use wayland_client::{protocol::wl_registry, Connection, EventQueue};
use wayland_protocols::wp::security_context::v1::client::wp_security_context_manager_v1::WpSecurityContextManagerV1;
use wayland_protocols::wp::security_context::v1::client::wp_security_context_v1::WpSecurityContextV1;

struct AppData {
    security_manager: Option<WpSecurityContextManagerV1>,
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    socket: PathBuf,

    #[arg(long)]
    close_fd: i32,

    #[arg(long)]
    app_id: Option<String>,

    #[arg(long)]
    sandbox_engine: Option<String>,

    #[arg(long)]
    instance_id: Option<String>,
}

impl AppData {
    pub fn new() -> Self {
        Self {
            security_manager: None,
        }
    }
}

impl wayland_client::Dispatch<WpSecurityContextV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &WpSecurityContextV1,
        _: <WpSecurityContextV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl wayland_client::Dispatch<WpSecurityContextManagerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &WpSecurityContextManagerV1,
        _: <WpSecurityContextManagerV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl wayland_client::Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        proxy: &wl_registry::WlRegistry,
        event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } if interface == "wp_security_context_manager_v1" => {
                eprintln!("binding context");
                let security_manager =
                    proxy.bind::<WpSecurityContextManagerV1, _, _>(name, version, qhandle, ());

                state.security_manager = Some(security_manager);
            }
            _ => (),
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut app_data = AppData::new();
    let connection = Connection::connect_to_env()?;
    let display = connection.display();
    let mut event_queue: EventQueue<AppData> = connection.new_event_queue();
    let queue_handle = event_queue.handle();
    let _ = display.get_registry(&queue_handle, ());

    event_queue.roundtrip(&mut app_data)?;

    let Some(security_context) = &mut app_data.security_manager else {
        eprintln!("failed to bind to security context manager");
        std::process::exit(1);
    };

    if args.socket.exists() {
        std::fs::remove_file(args.socket.as_path())?;
    }

    let socket = nix::sys::socket::socket(
        nix::sys::socket::AddressFamily::Unix,
        nix::sys::socket::SockType::Stream,
        nix::sys::socket::SockFlag::empty(),
        None,
    )?;

    let addr = nix::sys::socket::UnixAddr::new(args.socket.as_os_str().as_bytes())?;

    let backlog = nix::sys::socket::Backlog::new(10)?;

    nix::sys::socket::bind(socket.as_fd().as_raw_fd(), &addr)?;
    nix::sys::socket::listen(&socket.as_fd(), backlog)?;

    let security_context = security_context.create_listener(
        socket.as_fd(),
        unsafe { File::from_raw_fd(1).as_fd() },
        &queue_handle,
        (),
    );

    if let Some(app_id) = &args.app_id {
        security_context.set_app_id(app_id.clone());
    }

    if let Some(sandbox_engine) = &args.sandbox_engine {
        security_context.set_sandbox_engine(sandbox_engine.clone());
    }

    if let Some(instance_id) = &args.instance_id {
        security_context.set_instance_id(instance_id.clone());
    }

    eprintln!("opening connection");

    security_context.commit();

    connection.flush()?;

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1)
        }
    }
}
