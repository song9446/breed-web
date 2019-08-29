<script context="module">
import { createEventDispatcher } from 'svelte';

const press_duration = 0.5;
const press_range = 30;
let last_mouse_x = 0,
    last_mouse_y = 0,
    mouse_x = 0,
    mouse_y = 0,
    dragging = false,
    hold_timer = null,
    last_target_position = null,
    last_target_x = 0,
    last_target_y = 0,
    target = null;
function dragstart(e, callback){
    if(dragging) 
        dragend(e, callback);
    e.preventDefault();
    mouse_x = last_mouse_x = e.pageX;
    mouse_y = last_mouse_y = e.pageY;
    target = e.currentTarget;
    last_target_position = target.style.position || null;
    last_target_x = parseInt(target.getBoundingClientRect().left) || 0;
    last_target_y = parseInt(target.getBoundingClientRect().top) || 0;
    target.style.left = mouse_x - last_mouse_x + last_target_x + "px";
    target.style.top = mouse_y - last_mouse_y + last_target_y + "px";
    target.style.position = "fixed";
    let cloned = document.createElement("div");
    cloned.style.display = "inline-block";
    cloned.style.width = target.getBoundingClientRect().width + "px";
    cloned.style.height = target.getBoundingClientRect().height -4 + "px";
    target.parentNode.insertBefore(cloned, target);
    dragging = true;
    callback['dragstart'](target);
}
function dragend(e, callback){
    if(!dragging)
        return;
    e.preventDefault();
    dragging = false;
    target.style.left = last_target_x + "px";
    target.style.top = last_target_y + "px";
    target.style.position = last_target_position;
    console.log(target.previousElementSibling);
    target.previousElementSibling.remove();
    if(hold_timer) 
        clearTimeout(hold_timer);
    callback['dragend'](target);
}
function drag(e, callback){
    if(!dragging)
        return;
    e.preventDefault();
    if(e.touches && e.touches.length > 0){
        mouse_x = e.touches[0].pageX;
        mouse_y = e.touches[0].pageY;
    }
    else{
        mouse_x = e.pageX;
        mouse_y = e.pageY;
    }
    target.style.left = mouse_x - last_mouse_x + last_target_x + "px";
    target.style.top = mouse_y - last_mouse_y + last_target_y + "px";
    callback['drag'](target);
}

function mousedown(e, callback){
    dragstart(e, callback);
}
function mouseup(e, callback){
    dragend(e, callback);
}
function touchstart(e, callback){
    last_mouse_x = e.touches[0].pageX;
    last_mouse_y = e.touches[0].pageY;
    hold_timer = setTimeout(()=>{
        e.pageX = touch_mouse_x;
        e.pageX = touch_mouse_y;
        dragstart(e, callback);
    }, press_duration);
}
function touchend(e, callback){
    dragend(e, callback);
}
function touchmove(e, callback){
    if(dragging)
        return;
    mouse_x = e.touches[0].pageX
    mouse_y = e.touches[0].pageY;
    if(hold_timer && (touch_mouse_x < mouse_x-press_range || touch_mouse_x > mouse_x+press_range 
                      || touch_mouse_y < mouse_y-press_range || touch_mouse_y > mouse_y+press_range)){
        dragend(e, callback);
    }
}
</script>
<script>
const dispatch = createEventDispatcher();
let is_dragging = false;
function dragstart_callback(target){
    is_dragging = true;
    dispatch('dragstart', target);
}
function dragend_callback(target){
    is_dragging = false;
    console.log(target);
    dispatch('dragend', target);
}
function drag_callback(target){
    if(is_dragging)
    dispatch('drag', target);
}
let callback = {
    "dragstart": dragstart_callback,
    "dragend": dragend_callback,
    "drag": drag_callback,
};
</script>

<style>
div {
  display: inline-block;
}
div.is_dragging {
}
div:hover {
    filter:brightness(150%);
}
div:active:hover {
}
</style>

<svelte:window on:mouseup={e=>mouseup(e, callback)} on:touchend={e=>touchend(e, callback)} on:mousemove={e=>drag(e, callback)} on:touchmove={e=>drag(e, callback)} />

<div class:is_dragging on:mousedown={e=>mousedown(e, callback)} on:touchstart={e=>touchstart(e, callback)} on:touchmove={e=>touchmove(e, callback)}>
    <slot></slot>
</div>
