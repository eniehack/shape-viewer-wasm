import init, { parse_shp } from '$lib/wasm/shp_parser';

let initialized = false;

self.onmessage = async (e: MessageEvent<Uint8Array>) => {
	if (!initialized) {
		await init();
		initialized = true;
	}

	try {
		const result = parse_shp(e.data);
		self.postMessage({ success: true, data: result });
	} catch (error) {
		self.postMessage({ success: false, error: String(error) });
	}
};
