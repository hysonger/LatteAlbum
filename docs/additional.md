# Additional Information

## CI/CD

GitHub Actions workflow (`.github/workflows/ci.yml`):
- Triggered on push/PR to `main`
- Jobs: Build, Frontend, Backend (with vendored libheif + tests)

Local validation:
```bash
cd rust && ./cargo-with-vendor.sh test --features "vendor-build,video-processing"
cd frontend && npm install && npm run build
```
