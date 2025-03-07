# shortlink-rs

## Specs
[cf-short-link](https://github.com/AsenHu/cf-short-link)<br>
The behavior of this rust implementation **should** be the same as the upstream specs implementation.<br>
If there's any unexpect behavior, please create an issue.

## Disclaimer
This project is using [cloudflare worker-rs](https://github.com/cloudflare/workers-rs), the performace of it may or may-not be better than the original typescript implementation, your mileage might vary.

## Deploy
```bash
# Dependency
cargo install --locked worker-build
# Build
cp wrangler-example.toml wrangler.toml
npx wrangler deploy
```