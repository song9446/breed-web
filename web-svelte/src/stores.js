import { writable } from 'svelte/store';

export const history = (()=>{
    const { subscribe, set, update } = writable(0);
    return {
    }
})();
