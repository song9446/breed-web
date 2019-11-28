<script>
import { character_element_id_prefix, } from './Character.svelte';
import Character from './Character.svelte';
import Event from './Event.svelte';
import { app } from './app.js';
import { onMount, onDestroy } from 'svelte';

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
            console.log(res.message);
        else if(res.type == "events"){
            for(let event of res.events){
                add_and_apply_event(event);
            }
        }
    })
    .catch(res=>console.log(res))
}, 1000)

function id2character(id) {
    return characters_dict[id];
}

function marry(id1, id2) {
    console.log("try marry", id1, id2);
    app.marry(characters_dict[id1], characters_dict[id2])
    .then(res=>{
        if(res.type == "error")
            alert(res.message);
        else if(res.type == "event"){
            add_and_apply_event(res.event);
        }
    })
    .catch(res=>alert(res))
}

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
    if(!character_list) return;
    for(let ch of character_list){
        characters_dict[ch.id] = ch;
        ch["children"] = [];
    }
    if(family_tree_root == null){
        family_tree_root = character_list[0];
    }
    for(let ch of character_list){
        // only mather has children(for simplicity)
        if(ch.matherid){
            console.log("matherid:", ch.matherid);
            characters_dict[ch.matherid]["children"].push(ch);
            ch.mather = characters_dict[ch.matherid];
        }
        if(ch.fatherid){
            characters_dict[ch.fatherid]["children"].push(ch);
            ch.father = characters_dict[ch.fatherid];
        }
        if(ch.partnerid && ch.husband == null && ch.wife == null){
            ch.husband = characters_dict[ch.partnerid];
            characters_dict[ch.partnerid]["wife"] = ch;
        }
    }
    let root = null;
    for(let ch of character_list){
        if(ch != family_tree_root && !ch.matherid && !ch.fatherid && !ch.wife && !ch.husband && !(ch.children && ch.children.length))
            strangers = [...strangers, ch];
        else
            root = ch;
    }
    while(root) { //|| (root && root.matherid || root.fatherid)){
        if(root.matherid)
            root = characters_dict[root.matherid];
        else if(root.fatherid)
            root = characters_dict[root.fatherid];
        else if(root.wife)
            root = root.wife;
        else if(root.husband && (root.husband.matherid || root.husband.fatherid))
            root = root.husband;
        else
            break;
    }
    if(root) family_tree_root = root;
    console.log(characters_dict);
}

function add_and_apply_event(event) {
    if(event) console.log(event);
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
        event.related_element_ids = [character_element_id_prefix + event.summon.id];
        document.getElementById(character_element_id_prefix + event.summon.id + "-img").src = app.character_image_url(event.summon, "default");
        for(let i in event.summon){
            if(characters_dict[event.summon.id])
                characters_dict[event.summon.id][i] = event.summon[i];
        }
    break;
    case "pregnant":
        event.icon_src = "";
        event.name = "Pregnant";
        event.desc = "One of your family is pregnant.";
        event.related_element_ids = [
            character_element_id_prefix + event.fetus.id, 
            character_element_id_prefix + event.fetus.matherid];
        add_characters([event.fetus]);
    break;
    case "born":
        event.icon_src = "";
        event.name = "Born";
        event.desc = "A baby is born.";
        event.related_element_ids = [ character_element_id_prefix + event.baby.id ];
        document.getElementById(character_element_id_prefix + event.baby.id + "-img").src = app.character_image_url(event.baby, "default");
        for(let i in event.baby){
            if(characters_dict[event.baby.id])
                characters_dict[event.baby.id][i] = event.baby[i];
        }
    break;
    case "married":
        event.icon_src = "";
        event.name = "Married";
        event.desc = "One of your family married.";
        event.related_element_ids = [ 
            character_element_id_prefix + event.brideid,  
            character_element_id_prefix + event.groomid];
        characters_dict[event.brideid].partnerid = event.groomid;
        characters_dict[event.groomid].partnerid = event.brideid;
        characters_dict[event.brideid].husband = characters_dict[event.groomid];
        characters_dict[event.groomid].wife = characters_dict[event.brideid];
        strangers = strangers.filter(ch => ch.id != event.brideid && ch.id != event.groomid)
    }
    events = [event, ...events];
}

function character_dragstart(e) {
    let drag_target_id = parseInt(e.currentTarget.id.split(character_element_id_prefix)[1]);
    console.log("drag_start", drag_target_id);
    e.dataTransfer.setData("target_character_id", drag_target_id);
}
function character_drop(e) {
    e.preventDefault();
    let drag_target_id = e.dataTransfer.getData("target_character_id"),
        drop_target_id = parseInt(e.currentTarget.id.split(character_element_id_prefix)[1]);
    console.log("drag_end", e.currentTarget);
    marry(drag_target_id, drop_target_id);
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
    <Character character={family_tree_root} 
        on:dragstart={character_dragstart}
        on:drop={character_drop}/>
    {/if}
</section>
<section class="strangers">
    <h1>
        Strangers
    </h1>
    <ul>
        {#each strangers as stranger}
        <li>
            <Character character={stranger}
                on:dragstart={character_dragstart}
                on:drop={character_drop} />
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
