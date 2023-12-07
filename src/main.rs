use std::env;
use std::sync::Arc;

//mod picformat;

use axum::extract::State;
use axum::http;
use axum::routing::post;
use axum::Json;
use axum::Router;

use hyper::header;
use hyper::Request;
use serde::Deserialize;
use thirtyfour::prelude::*;

use tower::ServiceBuilder;
use tower::service_fn;
use tower_http::body::Full;
use tower_http::compression::Compression;
use tower_http::compression::predicate::SizeAbove;
use tower_http::compression::CompressionLayer;

#[derive(Deserialize)]
struct InForm {
    url: String,
    h: u32,
    w: u32,
    filename: String,
    fileformat: String,
}

struct Driver {
    pub driver: WebDriver,
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let driver = new_driver().await;
    let shared_state = Arc::new(Driver {
        driver: driver.clone(),
    });

    let _a = |req: &Request<Full>| {
        let _body = req.body();
    };

    let service = service_fn(|_: ()| async {
        Ok::<_, std::io::Error>(http::Response::new(()))
    });
    let middleware =
        ServiceBuilder::new().service(
            Compression::new(service)
                .compress_when(
                    SizeAbove::new(32)
                )
                .br(true)
                .gzip(true)
                .deflate(true)
                .quality(tower_http::CompressionLevel::Best)
                
        );

    let app = Router::new()
        .route("/api", post(take_pic))
        .layer(middleware)
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    driver.quit().await.unwrap();
}

async fn new_driver() -> WebDriver {
    let ip = env::args().nth(1).unwrap_or("localhost".to_owned());
    let port = env::args().nth(2).unwrap_or("9515".to_owned());
    let ds = format!("http://{ip}:{port}");

    let mut caps = DesiredCapabilities::chrome();

    caps.set_headless().unwrap();

    WebDriver::new(ds.as_str(), caps).await.unwrap()
}

async fn take_pic(
    State(state): State<Arc<Driver>>,
    Json(payload): Json<InForm>,
) -> impl axum::response::IntoResponse {
    let driver = state.driver.clone();

    driver
        .set_window_rect(0, 0, payload.h, payload.w)
        .await
        .unwrap();

    driver.goto(payload.url).await.unwrap();

    let a = driver.screenshot_as_png().await.unwrap();

    let headers = [
        //(header::CONTENT_ENCODING, "br"),
        (header::CONTENT_TYPE, "image/png"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"aa.png\"",
        ),
    ];

    println!("saved: {}", a.len());
    (headers, a)
    //driver.screenshot_as_png().await.unwrap()
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let ip = env::args().nth(1).unwrap_or("localhost".to_owned());
//     let port = env::args().nth(2).unwrap_or("9515".to_owned());
//     let ds = format!("http://{ip}:{port}");

//     let mut caps = DesiredCapabilities::chrome();
//     caps.add_chrome_option(
//         "prefs",
//         serde_json::json!({
//             "profile.default_content_settings": {
//                 "images": 2
//             },
//             "profile.managed_default_content_settings": {
//                 "images": 2
//             }
//         }),
//     )?;

//     //caps.add_chrome_arg("--force-device-scale-factor=0.5")?;
//     caps.add_chrome_arg("--window-size=2884,1560")?;
//     let _p = Proxy::Manual {
//         ftp_proxy: Option::None,
//         http_proxy: Option::None,
//         ssl_proxy: Option::None,
//         socks_proxy: Option::Some("127.0.0.1:10000".to_owned()),
//         socks_version: Option::Some(5),
//         socks_username: Option::None,
//         socks_password: Option::None,
//         no_proxy: Option::None,
//     };
//     //caps.set_proxy(p).unwrap();
//     caps.set_headless().unwrap();

//     dbg!(&ds);
//     let driver = WebDriver::new(ds.as_str(), caps).await?;
//     driver.set_window_rect(0, 0, 1000, 2000).await?;
//     let r = driver.get_window_rect().await.unwrap();
//     dbg!(r);
//     // Navigate to https://wikipedia.org.
//     driver.goto("https://taobao.com       ").await?;
//     driver.screenshot(Path::new("g.png")).await?;

//     let a: WindowHandle = driver.new_tab().await?;
//     let dd = driver.clone();
//     dd.switch_to_window(a).await?;
//     dd.goto("https://baidu.com").await?;
//     //driver.set_window_rect(0, 0, 2884, 1560).await?;

//     //dd.execute_async(r#"document.body.style.zoom='250%'"#, Vec::new()).await?;

//     dd.screenshot(Path::new("b.png")).await?;

//     println!("saved");

//     tokio::time::sleep(Duration::new(20, 0)).await;
//     // Always explicitly close the browser. There are no async destructors.
//     driver.quit().await?;
//     dd.quit().await?;
//     Ok(())
// }
