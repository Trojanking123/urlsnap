//! Requires chromedriver running on port 9515:
//!
//!     chromedriver --port=9515
//!
//! Run as follows:
//!
//!     cargo run --example chrome_options

use std::{path::Path, time::Duration};

use thirtyfour::{prelude::*, Proxy};

#[tokio::main]
async fn main() -> Result<(), WebDriverError> {
    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_option(
        "prefs",
        serde_json::json!({
            "profile.default_content_settings": {
                "images": 2
            },
            "profile.managed_default_content_settings": {
                "images": 2
            }
        }),
    )?;

    let _p = Proxy::Manual {
        ftp_proxy: Option::None,
        http_proxy: Option::None,
        ssl_proxy: Option::None,
        socks_proxy: Option::Some("127.0.0.1:10000".to_owned()),
        socks_version: Option::Some(5),
        socks_username: Option::None,
        socks_password: Option::None,
        no_proxy: Option::None,
    };
    //caps.set_proxy(p).unwrap();
    caps.set_headless().unwrap();

    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    driver.set_window_rect(0, 0, 1000, 2000).await?;
    let r = driver.get_window_rect().await.unwrap();
    dbg!(r);
    // Navigate to https://wikipedia.org.
    driver.goto("https://taobao.com       ").await?;
    driver.screenshot(Path::new("g.png")).await?;

    let a: WindowHandle = driver.new_tab().await?;
    let dd = driver.clone();
    dd.switch_to_window(a).await?;
    dd.goto("https://www.baidu.com").await?;
    driver.set_window_rect(0, 0, 2560 * 5, 1440 * 5).await?;

    //dd.execute_async(r#"document.body.style.zoom='250%'"#, Vec::new()).await?;

    dd.screenshot(Path::new("b.png")).await?;

    println!("saved");

    tokio::time::sleep(Duration::new(20, 0)).await;
    // Always explicitly close the browser. There are no async destructors.
    driver.quit().await?;
    dd.quit().await?;
    Ok(())
}
