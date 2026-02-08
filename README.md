## build

### wasm

1. install wasm-pack
1. `cd shp-parser && cargo install`
1. `wasm-pack build --target web --out-dir ../frontend/src/lib/wasm`

### frontend

1. build wasm
2. `pnpm run build`