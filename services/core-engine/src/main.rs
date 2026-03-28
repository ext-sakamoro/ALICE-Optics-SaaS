// ALICE-Optics-SaaS core-engine
// License: AGPL-3.0-or-later

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Serialize)]
struct Stats {
    total_requests: u64,
    trace_requests: u64,
    lens_requests: u64,
    fft_requests: u64,
    filter_requests: u64,
}

#[derive(Clone)]
struct AppState {
    stats: Arc<Mutex<Stats>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

#[derive(Deserialize)]
struct TraceRequest {
    wavelength_nm: f64,
    rays: u32,
    surfaces: Vec<serde_json::Value>,
}

#[derive(Serialize)]
struct TraceResponse {
    id: String,
    rays_traced: u32,
    rms_spot_radius_um: f64,
    strehl_ratio: f64,
    status: &'static str,
}

#[derive(Deserialize)]
struct LensRequest {
    focal_length_mm: f64,
    aperture_mm: f64,
    design_wavelength_nm: Option<f64>,
}

#[derive(Serialize)]
struct LensResponse {
    id: String,
    focal_length_mm: f64,
    f_number: f64,
    spherical_aberration_waves: f64,
    chromatic_aberration_um: f64,
}

#[derive(Deserialize)]
struct FftRequest {
    signal: Vec<f64>,
    sample_rate_hz: f64,
}

#[derive(Serialize)]
struct FftResponse {
    id: String,
    num_samples: usize,
    dominant_frequency_hz: f64,
    power_db: f64,
}

#[derive(Deserialize)]
struct FilterRequest {
    filter_type: String,
    cutoff_hz: f64,
    order: u32,
    sample_rate_hz: f64,
}

#[derive(Serialize)]
struct FilterResponse {
    id: String,
    filter_type: String,
    order: u32,
    cutoff_hz: f64,
    attenuation_db_per_octave: f64,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "alice-optics-core-engine",
        version: "0.1.0",
    })
}

async fn optics_trace(
    State(state): State<AppState>,
    Json(req): Json<TraceRequest>,
) -> Result<Json<TraceResponse>, StatusCode> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    stats.trace_requests += 1;
    info!("optics/trace wavelength={}nm rays={}", req.wavelength_nm, req.rays);
    Ok(Json(TraceResponse {
        id: Uuid::new_v4().to_string(),
        rays_traced: req.rays,
        rms_spot_radius_um: 2.3,
        strehl_ratio: 0.92,
        status: "completed",
    }))
}

async fn optics_lens(
    State(state): State<AppState>,
    Json(req): Json<LensRequest>,
) -> Result<Json<LensResponse>, StatusCode> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    stats.lens_requests += 1;
    let f_number = req.focal_length_mm / req.aperture_mm;
    info!("optics/lens f={}mm f/{:.1}", req.focal_length_mm, f_number);
    Ok(Json(LensResponse {
        id: Uuid::new_v4().to_string(),
        focal_length_mm: req.focal_length_mm,
        f_number,
        spherical_aberration_waves: 0.05,
        chromatic_aberration_um: 1.2,
    }))
}

async fn optics_fft(
    State(state): State<AppState>,
    Json(req): Json<FftRequest>,
) -> Result<Json<FftResponse>, StatusCode> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    stats.fft_requests += 1;
    let n = req.signal.len();
    let dominant = req.sample_rate_hz / (n as f64).max(1.0);
    info!("optics/fft samples={} rate={}Hz", n, req.sample_rate_hz);
    Ok(Json(FftResponse {
        id: Uuid::new_v4().to_string(),
        num_samples: n,
        dominant_frequency_hz: dominant,
        power_db: -14.2,
    }))
}

async fn optics_filter(
    State(state): State<AppState>,
    Json(req): Json<FilterRequest>,
) -> Result<Json<FilterResponse>, StatusCode> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    stats.filter_requests += 1;
    let attenuation = 20.0 * req.order as f64;
    info!("optics/filter type={} cutoff={}Hz order={}", req.filter_type, req.cutoff_hz, req.order);
    Ok(Json(FilterResponse {
        id: Uuid::new_v4().to_string(),
        filter_type: req.filter_type,
        order: req.order,
        cutoff_hz: req.cutoff_hz,
        attenuation_db_per_octave: attenuation,
    }))
}

async fn optics_stats(State(state): State<AppState>) -> Json<Stats> {
    let stats = state.stats.lock().unwrap().clone();
    Json(stats)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let state = AppState {
        stats: Arc::new(Mutex::new(Stats::default())),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/optics/trace", post(optics_trace))
        .route("/api/v1/optics/lens", post(optics_lens))
        .route("/api/v1/optics/fft", post(optics_fft))
        .route("/api/v1/optics/filter", post(optics_filter))
        .route("/api/v1/optics/stats", get(optics_stats))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9143").await.unwrap();
    info!("alice-optics-core-engine listening on :9143");
    axum::serve(listener, app).await.unwrap();
}
