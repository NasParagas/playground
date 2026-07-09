use axum::{Json, Router, extract::State, routing::post};
use serde::Deserialize;
use serde_json::Value;
use std::io::Write;
use std::sync::{Arc, Mutex};

mod hourglass;
use hourglass::Hourglass;

#[derive(Debug, Deserialize)]
struct Payload {
    payload: Vec<Sample>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Sample {
    name: String,
    time: u64,
    values: Values,
}

#[derive(Debug, Deserialize)]
struct Values {
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
}

type SharedHourglass = Arc<Mutex<Hourglass>>;

async fn receive(
    State(hourglass): State<SharedHourglass>,
    Json(body): Json<Payload>,
) -> &'static str {
    for sample in body.payload {
        if sample.name == "accelerometeruncalibrated" {
            if let (Some(x), Some(y), Some(z)) = (sample.values.x, sample.values.y, sample.values.z)
            {
                println!("accel  x={:+.3}  y={:+.3}  z={:+.3}", x, y, z);
            }
        } else if sample.name == "gravity" {
            if let Some(y) = sample.values.y {
                let mut hourglass = hourglass.lock().unwrap();
                hourglass.update(y, sample.time);
                // 画面をクリアして左上に描画し直すことで、その場で更新されているように見せる
                print!("\x1B[2J\x1B[H{}\n", hourglass.render());
                std::io::stdout().flush().unwrap();
            }
        } else {
            println!("{:?}", sample);
        }
    }
    "ok"
}

#[tokio::main]
async fn main() {
    // 砂が全て落ちきるまでの時間(分)
    const DURATION_MINUTES: f64 = 1.0;
    let hourglass: SharedHourglass = Arc::new(Mutex::new(Hourglass::new(DURATION_MINUTES)));
    let app = Router::new()
        .route("/data", post(receive))
        .with_state(hourglass);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("listening on :8000/data");
    axum::serve(listener, app).await.unwrap();
}
