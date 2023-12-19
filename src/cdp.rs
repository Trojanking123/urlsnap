use crate::error::*;

use crate::device::*;
use crate::snap::*;

use async_trait::async_trait;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use thirtyfour::extensions::cdp::ChromeDevTools;
use thirtyfour::prelude::*;
use tracing::info;

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize)]
pub struct UserAgent {
    pub userAgent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    acceptLanguage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform: Option<String>,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize)]
pub enum Orientation {
    portraitPrimary,
    portraitSecondary,
    landscapePrimary,
    landscapeSecondary,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize)]
pub struct ScreenOrientation {
    pub r#type: Orientation,
    pub angle: u32,
}

impl ScreenOrientation {
    pub fn new(is_landscape: bool, angel: u32) -> Self {
        let orien = if is_landscape {
            Orientation::landscapePrimary
        } else {
            Orientation::portraitPrimary
        };
        ScreenOrientation {
            r#type: orien,
            angle: angel,
        }
    }
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub width: u32,
    pub height: u32,
    pub deviceScaleFactor: f32,
    pub mobile: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenWidth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenHeight: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenOrientation: Option<ScreenOrientation>,
    #[serde(skip)]
    pub isTouchable: bool,
    #[serde(skip)]
    pub isDarkMode: bool,
    #[serde(skip)]
    pub userAgent: Option<UserAgent>,
}

#[allow(dead_code)]
impl DeviceInfo {
    pub fn new(w: u32, h: u32, sf: f32, is_mobile: bool) -> Self {
        DeviceInfo {
            width: w,
            height: h,
            deviceScaleFactor: sf,
            mobile: is_mobile,
            scale: None,
            screenWidth: None,
            screenHeight: None,
            screenOrientation: None,
            isTouchable: false,
            isDarkMode: false,
            userAgent: None,
        }
    }
}

impl From<&InForm> for DeviceInfo {
    fn from(value: &InForm) -> Self {
        let mut dev = DeviceInfo {
            width: value.w,
            height: value.h,
            deviceScaleFactor: value.factor.unwrap_or(1.0),
            mobile: false,
            scale: None,
            screenHeight: None,
            screenWidth: None,
            screenOrientation: Some(ScreenOrientation::new(value.landscape.unwrap_or(false), 0)),
            isTouchable: value.touch.unwrap_or(false),
            isDarkMode: value.darkmode.unwrap_or(false),
            userAgent: Some(UserAgent {
                userAgent: value.useragent.clone().unwrap_or("".to_owned()),
                acceptLanguage: value.lang.clone(),
                platform: value.platform.clone(),
            }),
        };

        if let Some(ref devid) = value.device {
            let dev_json = DEVICE_LIST.get().unwrap().get(devid).unwrap();
            dev.width = dev_json.viewport.width;
            dev.height = dev_json.viewport.height;
            dev.deviceScaleFactor = dev_json.viewport.deviceScaleFactor;
            dev.mobile = dev_json.viewport.isMobile;
            dev.screenOrientation = Some(ScreenOrientation::new(dev_json.viewport.isLandscape, 0));
            if let Some(ref mut ua) = dev.userAgent {
                if ua.userAgent.is_empty() {
                    ua.userAgent = dev_json.userAgent.clone();
                }
            }
        };

        //NOTICE: userAgent maybe null if both device and userAgent arm not set
        dev
    }
}

#[async_trait]
pub trait ChromeDevToolsOps {
    async fn call_cdp_command(&self, cmd: &str, params: Option<Value>)
        -> SnapResult<Option<Value>>;
    async fn get_browser_useragent(&self) -> SnapResult<String>;
    async fn set_request_useragent(&self, user_agent: &UserAgent) -> SnapResult<()>;
    async fn set_request_device(&self, device: &DeviceInfo) -> SnapResult<()>;
    async fn set_darkmode(&self, darkmode: bool) -> SnapResult<()>;
    async fn set_touchable(&self, touch: bool) -> SnapResult<()>;
    async fn set_scale_factor(&self, factor: f32) -> SnapResult<()>;
    async fn set_cookie_enabled(&self, enable: bool) -> SnapResult<()>;
    async fn set_scrollbar_hidden(&self, hidden: bool) -> SnapResult<()>;
}

#[async_trait]
impl ChromeDevToolsOps for ChromeDevTools {
    async fn call_cdp_command(
        &self,
        cmd: &str,
        params: Option<Value>,
    ) -> SnapResult<Option<Value>> {
        let res = match params {
            Some(params) => self.execute_cdp_with_params(cmd, params).await,
            None => self.execute_cdp(cmd).await,
        };
        match res {
            Ok(v) => Ok(Some(v)),
            Err(e) => match e {
                WebDriverError::Json(_) => Ok(None),
                _ => Err(SnapError::DriverError(e)),
            },
        }
    }

    async fn get_browser_useragent(&self) -> SnapResult<String> {
        let version_info = self
            .call_cdp_command("Browser.getVersion", None)
            .await?
            .unwrap();
        let user_agent = version_info["userAgent"].as_str().unwrap().to_owned();
        info!("user agent: {}", user_agent);
        Ok(user_agent)
    }

    async fn set_request_useragent(&self, user_agent: &UserAgent) -> SnapResult<()> {
        let value: Value = serde_json::to_value(user_agent)?;
        info!("send user agent: {:?}", value.to_string());
        self.call_cdp_command("Emulation.setUserAgentOverride", Some(value))
            .await?;
        Ok(())
    }

    async fn set_request_device(&self, device: &DeviceInfo) -> SnapResult<()> {
        let value: Value = serde_json::to_value(device)?;
        info!("send device: {:?}", value.to_string());

        self.call_cdp_command("Emulation.setDeviceMetricsOverride", Some(value))
            .await?;

        let args = json!({"enabled": device.mobile});
        self.call_cdp_command("Emulation.setTouchEmulationEnabled", Some(args))
            .await?;
        Ok(())
    }
    async fn set_darkmode(&self, darkmode: bool) -> SnapResult<()> {
        let args = json!({"enabled": darkmode});
        self.call_cdp_command("Emulation.setAutoDarkModeOverride", Some(args))
            .await?;
        Ok(())
    }

    async fn set_scale_factor(&self, factor: f32) -> SnapResult<()> {
        self.call_cdp_command("Emulation.resetPageScaleFactor", None)
            .await?;
        let args = json!({"pageScaleFactor": factor});
        self.call_cdp_command("Emulation.setPageScaleFactor", Some(args))
            .await?;

        Ok(())
    }

    async fn set_cookie_enabled(&self, enable: bool) -> SnapResult<()> {
        let args = json!({"enabled": enable});
        self.call_cdp_command("Emulation.setDocumentCookieDisabled", Some(args))
            .await?;
        Ok(())
    }

    async fn set_touchable(&self, touch: bool) -> SnapResult<()> {
        let args = json!({"enabled": touch});
        self.call_cdp_command("Emulation.setTouchEmulationEnabled", Some(args))
            .await?;
        Ok(())
    }

    async fn set_scrollbar_hidden(&self, hidden: bool) -> SnapResult<()> {
        let args = json!({"hidden": hidden});
        self.call_cdp_command("Emulation.setScrollbarsHidden", Some(args))
            .await?;
        Ok(())
    }
}
