# ALICE-Optics-SaaS

Optics and signal processing API — ray tracing, lens design, FFT analysis, and digital filter synthesis via the ALICE SaaS architecture.

## Architecture

```
Client
  └─ API Gateway (:8143) — JWT auth, rate limiting, proxy
       └─ Core Engine (:9143) — ray tracer, lens solver, FFT, filter design
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /health | Health check |
| POST | /api/v1/optics/trace | Ray tracing through optical system |
| POST | /api/v1/optics/lens | Lens design and aberration analysis |
| POST | /api/v1/optics/fft | Fast Fourier Transform on signal data |
| POST | /api/v1/optics/filter | Digital filter design and application |
| GET | /api/v1/optics/stats | Request statistics |

## Quick Start

```bash
cd services/core-engine && cargo run
cd services/api-gateway && cargo run
```

## License

AGPL-3.0-or-later
