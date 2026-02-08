<script lang="ts">
	import ShapeWorker from '$lib/shape_worker?worker';
	import { MapLibre, GeoJSONSource, CircleLayer } from 'svelte-maplibre-gl';
	import { onMount } from 'svelte';
	import type GeoJSON from '@types/geojson';

	let files = $state<FileList | undefined>();
	let geojson = $state<GeoJSON.FeatureCollection | undefined>();
	let worker: Worker | undefined = $state();
	let dialog: HTMLDialogElement = $state();
	let loading: boolean = $state(false);

	const onSubmitFile = async () => {
		if (!files || !worker || !dialog) {
			return;
		}
		console.debug("onSubmitFile");
		loading = true;
		const buf = await files[0].arrayBuffer();
		worker.postMessage(new Uint8Array(buf));
	};

	onMount(async () => {
		worker = new ShapeWorker();
		worker.onmessage = (e) => {
			if (e.data.success) {
				geojson = e.data.data;
			}
			loading = false;
			dialog.close();
		};

		dialog!.showModal();
	});
</script>

<dialog bind:this={dialog} class="rounded-xl p-6 backdrop:bg-black/40">
	<div class="p-2">
		<input bind:files class="block w-full text-sm file:mr-4 file:rounded-lg file:border-0 file:px-4 file:py-2" type="file" name="file" accept=".shp" />
	</div>
	<button class="rounded-lg bg-blue-600 px-6 py-2 text-white" type="button" onclick={() => onSubmitFile()}>parse</button>
</dialog>

<div class="fixed bottom-0 top-[53px]">
	<MapLibre class="h-full w-screen" style="https://basemaps.cartocdn.com/gl/voyager-gl-style/style.json">
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
</div>