<script>
import Character from './Character.svelte';
import { app } from './app.js';

export let gamedata = null;
let characters = {};
let strangers = [];
let family_tree_root = null;
let events = [];

$: if(gamedata) {
    console.log(gamedata);
    add_characters(gamedata.characters);
}

function add_characters(character_list){
    let root = null;
    console.log(character_list);
    for(let ch of character_list){
        characters[ch.id] = ch;
        ch["children"] = [];
    }
    for(let ch of character_list){
        // only mather has children(for simplicity)
        if(ch.matherid){
            characters[ch.matherid]["children"].push(ch);
            ch.mather = characters[ch.matherid];
        }
        if(ch.fatherid)
            ch.father = characters[ch.fatherid];
        if(ch.partnerid){
            ch.husband = characters[ch.partnerid];
            characters[ch.partnerid]["wife"] = ch;
        }
    }
    for(let ch of character_list){
        if(!ch.matherid && !ch.fatherid && !ch.wife && !ch.husband && !(ch.children && ch.children.length))
            strangers = [...strangers, ch];
        else
            root = ch;
    }
    while(root.matherid || root.fatherid){
        if(root.matherid)
            root = characters[root.matherid];
        else
            root = characters[root.fatherid];
    }
    family_tree_root = root;
}

</script>


<style>
div.container {
    display: grid;
    grid-template-areas:
        'events family strangers';
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
</div>
