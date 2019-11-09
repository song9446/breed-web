<script>
import Game from './Game.svelte';
import SessionManagePage from './SessionManagePage.svelte';
import { app } from './app.js';

let logined = false;
let user = null;
let characters = null;

function login(e){
    user = e.detail.user;
    characters = e.detail.characters;
    logined = true;
}

</script>


{#if logined }
<Game user={user} characters={characters}/>
{:else}
<SessionManagePage on:logined={login}/>
{/if}
