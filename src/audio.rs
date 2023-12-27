use anyhow::Result;
use com_policy_config::{IPolicyConfig, PolicyConfigClient};
use windows::{
    core::PCWSTR,
    Win32::{
        Devices::FunctionDiscovery::{PKEY_DeviceClass_IconPath, PKEY_Device_FriendlyName},
        Media::Audio::{
            eCommunications, eConsole, eMultimedia, eRender, IMMDevice, IMMDeviceEnumerator,
            MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        },
        System::Com::{
            CoCreateInstance, CoInitialize, StructuredStorage::PropVariantToStringAlloc,
            CLSCTX_ALL, STGM_READ,
        },
        UI::{
            Shell::{ExtractIconExW, PathParseIconLocationW},
            WindowsAndMessaging::HICON,
        },
    },
};

#[derive(Clone)]
pub(crate) struct AudioDevice(IMMDevice);

impl AudioDevice {
    fn get_id(&self) -> Result<PCWSTR> {
        unsafe { Ok(PCWSTR(self.0.GetId()?.0)) }
    }

    pub(crate) fn device_friendly_name(&self) -> Result<String> {
        unsafe {
            let store = self.0.OpenPropertyStore(STGM_READ)?;

            self.icon_handle()?;

            let variant = store.GetValue(&PKEY_Device_FriendlyName)?;
            let pwstr = PropVariantToStringAlloc(&variant)?;
            Ok(pwstr.to_string()?)
        }
    }

    pub(crate) fn icon_handle(&self) -> Result<HICON> {
        unsafe {
            let store = self.0.OpenPropertyStore(STGM_READ)?;

            let icon = store.GetValue(&PKEY_DeviceClass_IconPath)?;
            let icon_path = PropVariantToStringAlloc(&icon)?;
            let icon_index = PathParseIconLocationW(icon_path);

            let mut large = HICON::default();

            ExtractIconExW(PCWSTR(icon_path.0), icon_index, Some(&mut large), None, 1);

            Ok(large)
        }
    }
}

impl PartialEq for AudioDevice {
    fn eq(&self, other: &Self) -> bool {
        match (self.device_friendly_name(), other.device_friendly_name()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

pub(crate) struct AudioInterface {
    mm_device_enumerator: IMMDeviceEnumerator,
    policy_config: IPolicyConfig,
}

impl AudioInterface {
    pub(crate) fn new() -> Result<Self> {
        unsafe {
            CoInitialize(None)?;
            let mm_device_enumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let policy_config = CoCreateInstance(&PolicyConfigClient, None, CLSCTX_ALL)?;

            Ok(Self {
                mm_device_enumerator,
                policy_config,
            })
        }
    }

    pub(crate) fn get_default_output_device(&self) -> Result<AudioDevice> {
        unsafe {
            let mm_device = self
                .mm_device_enumerator
                .GetDefaultAudioEndpoint(eRender, eMultimedia)?;

            Ok(AudioDevice(mm_device))
        }
    }

    pub(crate) fn set_default_output_device(&self, device: &AudioDevice) -> Result<()> {
        unsafe {
            for role in [eConsole, eMultimedia, eCommunications] {
                self.policy_config
                    .SetDefaultEndpoint(device.get_id()?, role)?;
            }
            Ok(())
        }
    }

    pub(crate) fn get_output_devices(&self) -> Result<Vec<AudioDevice>> {
        unsafe {
            let state_mask = DEVICE_STATE_ACTIVE;
            let device_collection = self
                .mm_device_enumerator
                .EnumAudioEndpoints(eRender, state_mask)?;
            let count = device_collection.GetCount()?;
            Ok((0..count)
                .map(|i| {
                    device_collection
                        .Item(i)
                        .map(|mm_device| AudioDevice(mm_device))
                })
                .collect::<std::result::Result<_, _>>()?)
        }
    }
}
