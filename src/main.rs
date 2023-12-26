#![windows_subsystem = "windows"]

mod audio;
mod tray;

use anyhow::{anyhow, Result};
use audio::AudioDevice;
use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
};
use tray_icon::TrayIconEvent;

use crate::{audio::AudioInterface, tray::TrayManager};

const SPEAKERS: &str = "Speakers (High Definition Audio Device)";
const HEADPHONES: &str = "DELL S3422DWG (NVIDIA High Definition Audio)";

#[derive(PartialEq, Eq)]
pub(crate) enum OutputDevice {
    Speakers,
    Headphones,
    Unknown,
}

impl OutputDevice {
    pub(crate) fn icon_bytes(&self) -> &[u8] {
        let bytes = match self {
            OutputDevice::Speakers => include_bytes!("speakers.ico").as_slice(),
            OutputDevice::Headphones => include_bytes!("headphones.ico").as_slice(),
            _ => include_bytes!("unknown.ico").as_slice(),
        };

        bytes
    }
}

struct Handler {
    tray_manager: TrayManager,
    audio_interface: AudioInterface,
    speakers: AudioDevice,
    headphones: AudioDevice,
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

        let tray_manager = TrayManager::new()?;

        let handler = Self {
            tray_manager,
            audio_interface,
            speakers,
            headphones,
        };

        handler.update_icon()?;

        Ok(handler)
    }

    fn update_icon(&self) -> Result<()> {
        self.tray_manager.set_icon(self.output_device()?)?;

        Ok(())
    }

    fn output_device(&self) -> Result<OutputDevice> {
        let cur = self.audio_interface.get_default_output_device()?;

        let device = if cur == self.speakers {
            OutputDevice::Speakers
        } else if cur == self.headphones {
            OutputDevice::Headphones
        } else {
            OutputDevice::Unknown
        };

        Ok(device)
    }

    fn toggle(&self) -> Result<()> {
        let next = if self.output_device()? == OutputDevice::Speakers {
            &self.headphones
        } else {
            &self.speakers
        };

        self.audio_interface.set_default_output_device(next)?;
        self.update_icon()
    }
}

fn main() -> Result<()> {
    let event_loop: EventLoop<TrayIconEvent> = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    TrayIconEvent::set_event_handler(Some(move |e: TrayIconEvent| {
        proxy.send_event(e).expect("send event failed");
    }));

    let handler = Handler::new()?;

    event_loop.run(
        move |event, _, control_flow: &mut tao::event_loop::ControlFlow| {
            // println!("{event:?}, {:?}", std::thread::current().name());
            if let Event::UserEvent(_) = event {
                handler.toggle().expect("toggle failed");
            }
            *control_flow = ControlFlow::Wait;
        },
    );
}
