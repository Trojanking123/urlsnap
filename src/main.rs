use std::str::FromStr;
use std::sync::Arc;

mod error;
mod format_tran;
mod snap;

use crate::snap::*;

use error::SnapResult;
use format_tran::*;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Json;
use axum::Router;

use hyper::header;

use tower::ServiceBuilder;
use tower_http::compression::predicate::SizeAbove;
use tower_http::compression::CompressionLayer;

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

#[tokio::main]
async fn main() -> SnapResult<()> {
    // build our application with a single route
    let driver = new_driver().await?;
    let shared_state = Arc::new(Driver {
        driver: driver.clone(),
    });

    let middleware =
        ServiceBuilder::new().service(CompressionLayer::new().compress_when(SizeAbove::new(32)));

    let app = Router::new()
        .route("/api", post(take_pic_handler).get(take_pic_handler))
        .layer(middleware)
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    driver.quit().await?;
    Ok(())
}

async fn take_pic_handler(
    State(state): State<Arc<Driver>>,
    Json(payload): Json<InForm>,
) -> axum::response::Response {
    let driver = state.driver.clone();

    //let driver = new_driver().await;

    let trans_buffer = take_pic(driver, &payload).await.unwrap();

    let encoded = utf8_percent_encode(&payload.filename, NON_ALPHANUMERIC).to_string();

    let full_filname = format!("{}.{}", encoded, payload.fileformat.to_ascii_lowercase());
    let file_format = FileFormat::from_str(&payload.fileformat).unwrap();

    let dispostion = format!("attachment; filename=\"{}\"", full_filname);
    let c_type = get_content_type(file_format);
    let headers = [
        (header::CONTENT_TYPE, c_type),
        (header::CONTENT_DISPOSITION, dispostion),
    ];

    //driver.quit().await.?;
    println!("saved: {}", trans_buffer.len());
    (headers, trans_buffer).into_response()
}
