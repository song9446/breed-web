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

function logout(){
    logined = false;
    user = null;
    characters = null
}

</script>

<style>
.container {
    display: flex;
    flex-direction: column;
    height: 100%;
}
.topnav {
}
.main {
    flex-grow: 1;
    padding: 1rem;
}
</style>

<div class="container">
<SessionManagePage on:login={login} on:logout={logout} class="topnav" />
{#if logined }
<div class="main">
    <Game user={user} characters={characters}/>
</div>
{/if}
</div>
