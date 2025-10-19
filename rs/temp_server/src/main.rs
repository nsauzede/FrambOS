use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Result};
use chrono::Local;
use rand::Rng;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};
const DEVICE_FILE: &str = "/sys/bus/w1/devices/28-000001cda180/w1_slave";
struct AppState {
    temperature: Option<f32>,
}
fn format_date() -> String {
    let now = Local::now();
    now.format("%a %b %d %I:%M:%S %p %Z %Y").to_string()
}
#[get("/favicon.svg")]
async fn favicon(data: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let guard = data.lock().unwrap();
    let temperature = guard.temperature;
    let text = match temperature {
        Some(val) => format!("{:.1}", val),
        None => "--.-".to_string(),
    };
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
            <text x="50%" y="70%" text-anchor="middle"
                  font-size="32" font-family="monospace">{text}</text>
        </svg>"#
    );
    HttpResponse::Ok()
        .append_header(("Content-Type", "image/svg+xml"))
        .append_header(("Cache-Control", "no-store, no-cache, must-revalidate"))
        .body(svg)
}
#[get("/")]
async fn index(data: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let guard = data.lock().unwrap();
    let current_date = format_date();
    let formatted_temp = match guard.temperature {
        Some(temp) => format!("{:.3}&deg;C", temp),
        None => "--.-".to_string(),
    };
    let html = format!(
        r#"<!DOCTYPE html>
         <html>
         <head>
             <title>{formatted_temp}&deg;C on FrambOS</title>
             <meta http-equiv="refresh" content="10">
             <link rel="icon" type="image/svg+xml" href="/favicon.svg?t=0">
             <script>
setInterval(() => {{
    const link = document.querySelector("link[rel='icon']");
    link.href = `/favicon.svg?t=${{Date.now()}}`;
}}, 3000);
</script>
         </head>
         <body>
             <h1>Temperature on FrambOS</h1>
             On FrambOS right now, the date is &laquo;{}&raquo; and the temperature is {}
         </body>
         </html>"#,
        current_date, formatted_temp
    );
    HttpResponse::Ok().content_type("text/html").body(html)
}
async fn temperature_task(state: Arc<Mutex<AppState>>) {
    let mut timer = interval(Duration::from_secs(3));
    loop {
        timer.tick().await;
        let mut temperature = None;
        if let Ok(file) = File::open(DEVICE_FILE) {
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
            if lines.len() >= 2 && lines[0].ends_with("YES") {
                if let Some(temp_str) = lines[1].split("t=").nth(1) {
                    if let Ok(temp) = temp_str.trim().parse::<f32>() {
                        temperature = Some(temp / 1000.0);
                    }
                }
            }
        }
        if temperature == None {
            let mut rng = rand::rng();
            let succeed: bool = rng.random_bool(0.8);
            temperature = if succeed {
                Some(rng.random_range(15.0..30.0))
            } else {
                None
            };
        }
        let mut guard = state.lock().unwrap();
        guard.temperature = temperature;
    }
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let shared_state = Arc::new(Mutex::new(AppState { temperature: None }));
    {
        let state = shared_state.clone();
        tokio::spawn(async move {
            temperature_task(state).await;
        });
    }
    HttpServer::new({
        let state = shared_state.clone();
        move || {
            App::new()
                .app_data(web::Data::new(state.clone()))
                .service(index)
                .service(favicon)
        }
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    #[test]
    fn test_format_date() {
        let now = Local::now();
        let current_date = format_date();
        assert!(
            current_date.contains(now.format("%a").to_string().as_str()),
            "Failed to contain weekday: {}",
            current_date
        );
        assert!(
            current_date.contains(now.format("%b").to_string().as_str()),
            "Failed to contain month: {}",
            current_date
        );
        assert!(
            current_date.contains(now.format("%d").to_string().as_str()),
            "Failed to contain day: {}",
            current_date
        );
        assert!(
            current_date.contains(now.format("%Y").to_string().as_str()),
            "Failed to contain year: {}",
            current_date
        );
    }
}
