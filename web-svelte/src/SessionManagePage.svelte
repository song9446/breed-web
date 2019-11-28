<script>
import { app } from './app.js';
import { createEventDispatcher } from 'svelte';

const dispatch = createEventDispatcher();

let state = "login";

let user = null;

app.reload_session()
    .then(res => {
        if(res.type == "error")
            console.log(res.message);
        else {
            state = "logined";
            user = res.user;
            return dispatch('login', res);
        }
    });

function login(){
    let id = document.getElementById("loginbox-id").value,
        pw = document.getElementById("loginbox-pw").value;
    app.login(id, pw)
    .then(res => {
        if(res.type == "error")
            alert(res.message);
        else {
            state = "logined";
            user = res.user;
            return dispatch('login', res);
        }
    })
    .catch(res => alert(res));
}
function logout(){
    app.logout()
    .then(res => {
        if(res.type == "error")
            alert(res.message);
        else {
            state = "login"
            return dispatch('logout');
        }
    })
    .catch(res => alert(res));
}
function join_form_open(){
    state = "join";
}
function join(){
    let id = document.getElementById("loginbox-id").value,
        pw = document.getElementById("loginbox-pw").value,
        email = document.getElementById("loginbox-email").value,
        surname = document.getElementById("loginbox-surname").value;
    app.join(id, pw, surname, email)
    .then(res => {
        if(res.type == "error")
            alert(res.message);
        else {
            alert("thank you for join us!");
            state = "login"
        }
    })
    .catch(res => alert(res));
}

</script>

<style>
.fullscreen {
    display: flex;
    position: fix;
    height: 100%;
    width: 100%;
    align-items: center;
    justify-content: center;
}
.loginbox {
    flex-direction: column;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2em;
    border: 1px solid #aaa;
    border-radius: 12.5%;
    box-shadow: 2px 2px 8px rgba(0,0,0,0.5);
    background-color: white;
}
button {
    display: block;
    margin-top: 1.0em;
    width: 100%;
}
label {
    margin-left: 0.25em;
}
.title{
    text-align: center;
}

.topnav {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid #aaa;
    background-color: white;
    padding: 0.5rem;
}
.topnav .container {
    display: flex;
    align-items: center;
}
.topnav .container * {
    margin: 0;
    margin-left: 0.5rem;
    margin-right: 0.5rem;
    flex-shrink: 0;
    width: auto;
}
.topnav .title {
    font-size: 1.2rem;
}
</style>

{#if state=="logined" }
<div class="topnav">
    <div class="title">
        Degeneration
    </div>
    <div class="container">
        <span>
           Hi, <b>{user.nickname}</b>!
        </span>
        <button on:click={logout}>Logout</button>
    </div>
</div>

{:else}
<div class="fullscreen">
<div class="loginbox">
    <div class="title">
        <h1>
            Breed.moe
        </h1>
    </div>
    <div>
        <label for="loginbox-id">ID</label>
        <input id="loginbox-id" required />
    </div>
    <div>
        <label for="loginbox-pw">PW</label>
        <input id="loginbox-pw" type="password" required />
    </div>
    {#if state == "login"}
    <button on:click={login}>Login</button>
    <button on:click={join_form_open}>Join</button>
    {/if}
    {#if state == "join"}
    <div>
        <label for="loginbox-surname">Nickname</label>
        <input id="loginbox-surname" required />
    </div>
    <div>
        <label for="loginbox-email">Email</label>
        <input id="loginbox-email" required />
    </div>
    <button on:click={join}>Join</button>
    {/if}
</div>
</div>
{/if}
