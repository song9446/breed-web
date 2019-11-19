<script>
import { character_element_id_prefix, } from './Character.svelte';
import Character from './Character.svelte';
import Event from './Event.svelte';
import { app } from './app.js';

export let characters;
export let user;
let characters_dict = {};
let strangers = [];
let family_tree_root = null;
let events = [];

/*$: if(characters) {
    add_characters(characters);
}*/
add_characters(characters);

setInterval(()=>{
    let user_mana_updated = app.update_mana(user);
    user.mana = user_mana_updated.mana;
    user.mana_updated_at = user_mana_updated.mana_updated_at;
}, 100)

setInterval(()=>{
    app.update()
    .then(res=>{
        if(res.type == "error")
            alert(res.message);
        else if(res.type == "events"){
            console.log("update:", res.event);
            for(let event of res.events){
                add_and_apply_event(event);
            }
        }
    })
    .catch(res=>alert(res))
}, 1000)

function summon_character() {
    app.summon_character(user)
    .then(res=>{
        if(res.type == "error")
            alert(res.message);
        else if(res.type == "event"){
            add_and_apply_event(res.event);
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

function add_and_apply_event(event) {
    switch(event.name){
    case "summon_start":
        event.icon_src = "";
        event.name = "Summoning..";
        event.desc = "You just started a ritual of summoning."
        event.related_element_ids = [character_element_id_prefix + event.summon.id];
        add_characters([event.summon]);
        user.mana -= user.summon_mana_cost;
    break;
    case "summon_finish":
        event.icon_src = "";
        event.name = "Summonned";
        event.desc = "You finished a ritual of summoning. Somebody was teleported here from far way."
        event.related_element_ids = [character_element_id_prefix + event.summonid];
    break;
    case "pregnant":
        event.icon_src = "";
        event.name = "Pregnant";
        event.desc = "One of your family is pregnant.";
        event.related_element_ids = [
            character_element_id_prefix + event.fetus.id, 
            character_element_id_prefix + event.fetus.matherid];
    break;
    case "born":
        event.icon_src = "";
        event.name = "Born";
        event.desc = "A baby is born.";
        event.related_element_ids = [ character_element_id_prefix + event.fetus.id ];
    break;
    }
    events = [event, ...events];
}

</script>


<style>
div.container {
    display: grid;
    grid-template-areas:
        'events family strangers'
        'menu menu menu';
    grid-template-columns: min-content 1fr min-content;
    grid-template-rows: 1fr min-content;
    background-color: white;
    height: 100%;
    padding: 1rem;
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
section.menu {
    grid-area: menu;
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

.user-mana-container {
    display: flex;
    flex-direction: column;
}
.user-mana {
    display: flex;
    flex-direction: column;
}
.user-summon-mana-cost {
    display: flex;
    flex-direction: column;
}

.summon-button {
    font-size: 2rem;
}
</style>

<div class="container">
<section class="events">
    <h1>
        Events
    </h1>
    <ul>
        {#each events as event}
        <Event event={event} />
        {/each}
    </ul>
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
        <div class="user-mana"> 
            <span>마나</span> 
            <span>{user.mana.toFixed()}</span> 
        </div>
        <div class="user-summon-mana-cost"> 
            <span>소환에 필요한 마나</span> 
            <span>{user.summon_mana_cost} </span>
        </div>
    </div>
</section>
</div>
