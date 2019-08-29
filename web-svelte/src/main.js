import App from './App.svelte';

const app = new App({
	target: document.body,
	props: {
		name: 'world',
        data: {
            server_url: "breed.moe",
        },
	}
});

export default app;
