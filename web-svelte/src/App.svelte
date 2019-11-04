<script>
import Game from './Game.svelte';
import SessionManagePage from './SessionManagePage.svelte';
import { app } from './app.js';

let logined = false;
let gamedata = null;

function login(e){
    gamedata = e.detail.gamedata;
    logined = true;
}

</script>


{#if logined }
<Game gamedata={gamedata}/>
{:else}
<SessionManagePage on:logined={login}/>
{/if}
