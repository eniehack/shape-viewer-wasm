WASM_DIR = shp-parser
WEB_DIR = frontend

.PHONY: wasm web build clean prepare

prepare:
	pnpm install --frozen-lockfile

wasm:
	pnpm exec wasm-pack build $(WASM_DIR) --target web --release --out-dir ./$(WEB_DIR)/src/lib/wasm

web:
	cd ./$(WEB_DIR) \
	&& pnpm install --frozen-lockfile \
	&& pnpm build

build: prepare wasm web

clean:
	rm -rf $(WASM_DIR)/target
	rm -rf $(WEB_DIR)/src/lib/wasm
	rm -rf $(WEB_DIR)/.svelte-kit
	rm -rf $(WEB_DIR)/node_modules
