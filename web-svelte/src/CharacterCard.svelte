<script>
import { character_element_id_prefix } from './Character.svelte';
import { app } from './app.js';
export let character;
function dragover(e){
    e.dataTransfer.dropEffect = 'move';
}
</script>
<style>
ul {
  list-style-type: none;
  padding: 0;
  margin: 0;
}
img {
   min-width: 32px;
   max-width: 128px;
   min-height: 32px;
   max-height: 128px;
   height: 10vw;
   width: 10vw;
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
</style>

<ul class:highlight={false} class="character-properties" id="{character_element_id_prefix}{character.id}"  
    draggable="true"
    on:dragstart
    on:drop
    on:dragover|preventDefault={dragover}>
    <!--<li><img draggable="false" src={app.character_image_url(character, "default")} alt="loading.." style="width:{width}px; height:{height}px"/></li>-->
        <li ondrop="return false"
            ondragover="return false">
            <img draggable="false" 
            id="{character_element_id_prefix}{character.id}-img"
            src={app.character_image_url(character, "default")} 
            alt="loading.."/>
        </li>
        <li ondrop="return false"
            ondragover="return false">
            {character.firstname} ({character.gender? "♂️": "♀️"}) 
            <strong>{character.surname || ""}</strong>
        </li>
</ul>
