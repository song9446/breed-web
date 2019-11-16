<script>
import Character from './Character.svelte';
import { app } from './app.js';

export let characters
export let user = {};
let characters_dict = {};
let strangers = [];
let family_tree_root = null;
let events = [];

$: if(characters) {
    add_characters(characters);
}

setInterval(()=>{
    let user_mana_updated = app.update_mana(user);
    user.mana = user_mana_updated.mana;
    user.mana_updated_at = user_mana_updated.mana_updated_at;
}, 100)

function summon_character() {
    app.summon_character(user)
    .then(res=>{
        if(res.error)
            alert(res.error.message);
        else {
            console.log(res.data);
            add_characters([res.data.character]);
            user.mana = res.data.user.mana;
            user.mana_updated_at = res.data.user.updated_at;
        }
    })
    .catch(res=>alert(res));
}

function add_characters(character_list){
    let root = null;
    console.log(character_list);
    for(let ch of character_list){
        characters_dict[ch.id] = ch;
        ch["children"] = [];
    }
    for(let ch of character_list){
        // only mather has children(for simplicity)
        if(ch.matherid){
            characters_dict[ch.matherid]["children"].push(ch);
            ch.mather = characters_dict[ch.matherid];
        }
        if(ch.fatherid)
            ch.father = characters_dict[ch.fatherid];
        if(ch.partnerid){
            ch.husband = characters_dict[ch.partnerid];
            characters_dict[ch.partnerid]["wife"] = ch;
        }
    }
    for(let ch of character_list){
        if(!ch.matherid && !ch.fatherid && !ch.wife && !ch.husband && !(ch.children && ch.children.length))
            strangers = [...strangers, ch];
        else
            root = ch;
    }
    while(root && (root.matherid || root.fatherid)){
        if(root.matherid)
            root = characters_dict[root.matherid];
        else
            root = characters_dict[root.fatherid];
    }
    family_tree_root = root;
}

</script>


<style>
div.container {
    display: grid;
    grid-template-areas:
        'events family strangers'
        'menu menu menu';
    grid-template-columns: min-content 1fr min-content;
    background-color: white;
}
section.events {
    text-align: center;
    grid-area: events;
}
section.family {
    text-align: center;
    grid-area: family;
}
section.strangers {
    text-align: center;
    grid-area: strangers;
}
section menu {
    grid-area: summon_button;
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-direction: horizontal;
}
section other {
    text-align: center;
    grid-area: summon_button
}
.strangers ul{
  list-style-type: none;
  padding: 0;
  margin: 0;
}
.strangers ul>li{
  margin: 0.5em;
}
</style>

<div class="container">
<section class="events">
    <h1>
        Events
    </h1>
</section>
<section class="family">
    <h1>
        Family
    </h1>
    {#if family_tree_root}
    <Character character={family_tree_root} />
    {/if}
</section>
<section class="strangers">
    <h1>
        Strangers
    </h1>
    <ul>
        {#each strangers as stranger}
        <li>
            <Character character={stranger} />
        </li>
        {/each}
    </ul>
</section>
<section class="menu">
    <div></div>
    <div class="summon-button-container">
        <button class="summon-button" on:click={summon_character}>소환</button>
    </div>
    <div class="user-mana-container">
        <div class="user-mana"> {user.mana} </div>
        <div class="user-summon-mana-cost"> {user.summon_mana_cost} </div>
    </div>
</section>
</div>
