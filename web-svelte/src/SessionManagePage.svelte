<script>
import { app } from './app.js';
import { createEventDispatcher } from 'svelte';

const dispatch = createEventDispatcher();

let state = "login";

app.reload_session()
    .then(res => {
        if(res.error)
            console.log(res.error.message);
        else {
            return dispatch('logined', res.data);
        }
    });
function login(){
    let id = document.getElementById("loginbox-id").value,
        pw = document.getElementById("loginbox-pw").value;
    app.login(id, pw)
    .then(res => {
        if(res.error)
            alert(res.error.message);
        else {
            return dispatch('logined', res.data);
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
        if(res.error)
            alert(res.error.message);
        else {
            alert("thank you for join us!");
            state = "login"
        }
    })
    .catch(res => alert(res));
}

</script>

<style>
.container {
    display: flex;
    height: 100%;
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
</style>

<div class="container">
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
