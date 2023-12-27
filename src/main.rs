#![windows_subsystem = "windows"]

mod audio;
mod tray;

use std::{collections::HashMap, process};

use anyhow::{anyhow, Result};
use audio::AudioDevice;
use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
};
use tray_icon::{
    menu::{MenuEvent, MenuId},
    ClickType, Icon, TrayIconEvent,
};

use crate::{audio::AudioInterface, tray::TrayManager};

const SPEAKERS: &str = "Speakers (High Definition Audio Device)";
const HEADPHONES: &str = "DELL S3422DWG (NVIDIA High Definition Audio)";

struct Handler {
    tray_manager: TrayManager,
    audio_interface: AudioInterface,
    selected_devices: Vec<AudioDevice>,
    all_devices: HashMap<MenuId, AudioDevice>,
}

impl Handler {
    fn new() -> Result<Self> {
        let audio_interface = AudioInterface::new()?;

        let headphones = audio_interface
            .get_output_devices()?
            .into_iter()
            .filter(|x| {
                x.device_friendly_name()
                    .map(|name| name == HEADPHONES)
                    .unwrap_or(false)
            })
            .next()
            .ok_or(anyhow!("Headphones not found"))?;

        let speakers = audio_interface
            .get_output_devices()?
            .into_iter()
            .filter(|x| {
                x.device_friendly_name()
                    .map(|name| name == SPEAKERS)
                    .unwrap_or(false)
            })
            .next()
            .ok_or(anyhow!("Speakers not found"))?;

        let selected_devices = vec![headphones, speakers];

        let tray_manager = TrayManager::new()?;

        let mut handler = Self {
            tray_manager,
            audio_interface,
            selected_devices,
            all_devices: HashMap::default(),
        };

        handler.update()?;

        Ok(handler)
    }

    fn update_icon(&self) -> Result<()> {
        let cur = self.audio_interface.get_default_output_device()?;

        self.tray_manager
            .set_icon(Icon::from_handle(cur.icon_handle()?.0))?;

        Ok(())
    }

    fn next_device(&mut self) -> Result<()> {
        if self.selected_devices.len() > 0 {
            let cur = self.audio_interface.get_default_output_device()?;

            let index: usize = self
                .selected_devices
                .iter()
                .position(|x| x == &cur)
                .unwrap_or(0);
            let next = (index + 1) % self.selected_devices.len();

            self.audio_interface
                .set_default_output_device(&self.selected_devices[next])?;

            self.update()?;
        }

        Ok(())
    }

    fn update_tray_menu(&mut self) -> Result<()> {
        let devices = self.audio_interface.get_output_devices()?;

        self.all_devices = devices
            .into_iter()
            .enumerate()
            .map(|(i, d)| (MenuId::from(i), d))
            .collect();

        let mut devices = self
            .all_devices
            .iter()
            .map(|(id, d)| {
                d.device_friendly_name()
                    .map(|name| (name, self.selected_devices.contains(&d), id.to_owned()))
            })
            .collect::<Result<Vec<(String, bool, MenuId)>>>()?;
        devices.sort_unstable();
        devices.reverse();

        self.tray_manager.update_check_menu(devices)?;

        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        self.update_icon()?;
        self.update_tray_menu()?;

        Ok(())
    }

    fn handle(&mut self, event: TrayMenuEvent) -> Result<()> {
        println!("{event:?}");
        match event {
            TrayMenuEvent::TrayIconEvent(event) => {
                if event.click_type == ClickType::Left {
                    self.next_device().expect("next device failed");
                }
            }
            TrayMenuEvent::MenuEvent(event) => {
                if event.id.0 == "exit" {
                    process::exit(0);
                }

                let d = &self.all_devices[&event.id];
                if let Some(index) = self.selected_devices.iter().position(|x| x == d) {
                    self.selected_devices.remove(index);
                } else {
                    self.selected_devices.push(d.clone());
                }

                self.update()?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum TrayMenuEvent {
    TrayIconEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
}

fn main() -> Result<()> {
    let event_loop: EventLoop<TrayMenuEvent> = EventLoopBuilder::with_user_event().build();

    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |e: TrayIconEvent| {
        proxy
            .send_event(TrayMenuEvent::TrayIconEvent(e))
            .expect("send event failed");
    }));

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
        proxy
            .send_event(TrayMenuEvent::MenuEvent(e))
            .expect("send event failed");
    }));

    let mut handler = Handler::new()?;

    event_loop.run(
        move |event, _, control_flow: &mut tao::event_loop::ControlFlow| {
            // println!("{event:?}, {:?}", std::thread::current().name());
            if let Event::UserEvent(event) = event {
                handler.handle(event).expect("handle failed");
            }
            *control_flow = ControlFlow::Wait;
        },
    );
}
