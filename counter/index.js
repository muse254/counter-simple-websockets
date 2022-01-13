import init from './pkg/counter.js';

window.addEventListener('load', async () => {
    await init('./pkg/counter_bg.wasm');
});