<script lang="ts">
	import init, { greet, parse_shp } from '$lib/wasm/shp_parser';
	import { MapLibre, GeoJSONSource, CircleLayer } from 'svelte-maplibre-gl';
	import { onMount } from 'svelte';
	import type GeoJSON from '@types/geojson';

	let files = $state<FileList | undefined>();
	let initiated = $state(false);
	let geojson = $derived.by<GeoJSON.FeatureCollection | undefined>(async () => {
		if (!initiated || !files) return undefined;
		for (const file of files) {
			const buf = await file.arrayBuffer();
			const bytes = new Uint8Array(buf);
			return JSON.parse(JSON.stringify(parse_shp(bytes))) as GeoJSON.FeatureCollection;
		}
	});

	onMount(async () => {
		await init();
		initiated = true;
	});
</script>

<button onclick={() => greet()}>press it</button>

<input bind:files type="file" name="file" accept=".shp" />

<MapLibre class="h-[400px]" style="https://basemaps.cartocdn.com/gl/voyager-gl-style/style.json">
	{#if typeof geojson !== 'undefined'}
		<GeoJSONSource data={geojson}>
			<CircleLayer
				paint={{
					'circle-color': '#00ff00',
					'circle-radius': 8
				}}
			/>
		</GeoJSONSource>
	{/if}
</MapLibre>
