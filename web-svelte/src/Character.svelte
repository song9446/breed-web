<script context="module">
import { writable } from 'svelte/store';
import { app } from './app.js';
export const character_element_id_prefix = "character";
export const couple_element_id_prefix = "couple";
const path_element_id_prefix = "path";

let line_updaters = {};

setInterval(()=>{
    for(let id in line_updaters){
        line_updaters[id]();
    }
},);

</script>
<script>

import { fade, fly } from 'svelte/transition';
import { onMount } from 'svelte';
import Draggable from './Draggable.svelte';
import CharacterCard from './CharacterCard.svelte';

export let character = {
    "id": -1,
    "firstname": "error?",
    "surname": "",
    "imageurl": "http://www.pngall.com/wp-content/uploads/2/Question-Mark-PNG-Picture.png",
    "partnerid": 2,
};
let children_line_start = [0,0];
let children_line_ends = [];
let partner_line_start = null;
let partner_line_end = null;
function update_line_variables(){
    let ch = document.getElementById(couple_element_id_prefix + character.id),
        rect = ch.getBoundingClientRect();
    children_line_start = [rect.left + rect.width*0.5, rect.top + rect.height*0.5 + window.scrollY, character.id];
    children_line_ends = character.children.map(ch=>{
        let rect = document.getElementById(character_element_id_prefix + ch.id).getBoundingClientRect();
        return [rect.left + rect.width*0.5, rect.top + rect.height*0.5 + window.scrollY, ch.id];
    });
    if(character.husband != null || character.wife != null){
        let partner = character.husband || character.wife;
        let ch1 = document.getElementById(character_element_id_prefix + character.id),
            rect1 = ch1.getBoundingClientRect(),
            ch2 = document.getElementById(character_element_id_prefix + partner.id),
            rect2 = ch2.getBoundingClientRect();
        partner_line_start = [rect1.left + rect1.width*0.5, rect1.top + rect1.height*0.5 + window.scrollY, character.id];
        partner_line_end = [rect2.left + rect2.width*0.5, rect2.top + rect2.height*0.5 + window.scrollY, partner.id];
        children_line_start = [(partner_line_start[0]+partner_line_end[0])*0.5, 
                               (partner_line_start[1]+partner_line_end[1])*0.5, character.id]; 
    }
}
let mounted = false;
onMount(async () => mounted = true);
$: if(mounted) {
    update_line_variables()
}


</script>


<style context="module">
ul {
  list-style-type: none;
  padding: 0;
  margin: 0;
}
div.character {
  display: flex;
  flex-direction: column;
  align-items: center;
}
ul.character-parents {
  display: flex;
  flex-direction: row;
  z-index:1;
}
ul.character-parents>li:nth-child(2){
  margin-left: 3vw;
}
ul.character-children {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  margin-top: 1em;
  justify-content: center;
}
ul.character-children>li{
  margin: 1.5vw;
}

ul.character-properties {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  border: 1px solid #aaa;
  border-radius: 12.5%;
  box-shadow: 2px 2px 8px rgba(0,0,0,0.5);
  background-color: white;
  overflow: hidden;
  min-width: 32px;
  max-width: 128px;
  width: 10vw;
}
ul.character-properties.highlight {
  border: 2px solid red;
}

img {
   min-width: 32px;
   max-width: 128px;
   min-height: 32px;
   max-height: 128px;
   height: 10vw;
   width: 10vw;
}
svg {
    position: absolute;
    top: 0;
    left: 0;
    overflow: visible;
}
svg.overlap {
}
svg.underlap {
}
path {
  stroke-dasharray: 10;
  animation: dash 1s linear infinite;
}
path.highlight {
    stroke: red;
}
@keyframes dash {
  to {
    stroke-dashoffset: -20;
  }
}

</style>

<svelte:window on:resize={update_line_variables}/>

<div class="character">
    <!--<svg class="overlap">
        <g fill="none" stroke="black" stroke-width="1" stroke-linecap="none">
            {#each children_line_ends as line_end}
            <path d="M{children_line_start[0]},{children_line_start[1]} C{children_line_start[0]},{line_end[1]+30} {line_end[0]},{children_line_start[1]+60} {line_end[0]},{line_end[1]}"/>
            {/each}
        </g>
        {#if partner_line_start}
        <g fill="none" stroke="black" stroke-width="1" stroke-linecap="none">
            <path d="M{partner_line_start[0]},{partner_line_start[1]} {(partner_line_start[0]+partner_line_end[0])*0.5},{(partner_line_start[1]+partner_line_end[1])*0.5}"/>
            <path d="M{partner_line_end[0]},{partner_line_end[1]} {(partner_line_start[0]+partner_line_end[0])*0.5},{(partner_line_start[1]+partner_line_end[1])*0.5}"/>
        </g>
        {/if}
    </svg>-->
    <svg class="underlap">
        <g fill="none" stroke="white" stroke-width="6" stroke-linecap="none">
            {#each children_line_ends as line_end}
            <path 
                d="M{children_line_start[0]},{children_line_start[1]} C{children_line_start[0]},{line_end[1]} {line_end[0]},{children_line_start[1]} {line_end[0]},{line_end[1]}"/>
            {/each}
        </g>
        <g fill="none" stroke="black" stroke-width="4" stroke-linecap="none">
            {#each children_line_ends as line_end}
            <path 
                d="M{children_line_start[0]},{children_line_start[1]} C{children_line_start[0]},{line_end[1]} {line_end[0]},{children_line_start[1]} {line_end[0]},{line_end[1]}"/>
            {/each}
        </g>
        {#if partner_line_start}
        <g fill="none" stroke="white" stroke-width="6" stroke-linecap="none">
            <path 
                d="M{partner_line_start[0]},{partner_line_start[1]} {(partner_line_start[0]+partner_line_end[0])*0.5},{(partner_line_start[1]+partner_line_end[1])*0.5}"/>
            <path 
                d="M{partner_line_end[0]},{partner_line_end[1]} {(partner_line_start[0]+partner_line_end[0])*0.5},{(partner_line_start[1]+partner_line_end[1])*0.5}"/>
        </g>
        <g fill="none" stroke="black" stroke-width="4" stroke-linecap="none">
            <path 
                d="M{partner_line_start[0]},{partner_line_start[1]} {(partner_line_start[0]+partner_line_end[0])*0.5},{(partner_line_start[1]+partner_line_end[1])*0.5}"/>
            <path 
                d="M{partner_line_end[0]},{partner_line_end[1]} {(partner_line_start[0]+partner_line_end[0])*0.5},{(partner_line_start[1]+partner_line_end[1])*0.5}"/>
        </g>
        {/if}
    </svg>

    <ul class="character-parents" id="{couple_element_id_prefix}{character.id}">
        <li>
            <CharacterCard character={character}
                on:dragstart
                on:drop />
                <!--<li><img draggable="false" src={app.character_image_url(character, "default")} alt="loading.." style="width:{width}px; height:{height}px"/></li>-->
        </li>
        {#if character.husband}
        <li>
            <CharacterCard character={character.husband}
                on:dragstart
                on:drop />
        </li>
        {/if}
        {#if character.wife}
        <li>
            <CharacterCard character={character.wife}
                on:dragstart
                on:drop />
        </li>
        {/if}
    </ul>
    {#if character.children && character.children.length}
    <ul class="character-children">
        {#each character.children as child}
        <li>
            <svelte:self character={child} 
                on:dragstart
                on:drop />
        </li>
        {/each}
    </ul>
    {/if}
</div>
