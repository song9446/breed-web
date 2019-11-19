<script context="module">
</script>
<script>
export let event = null;
function highlight_related_elements(){
    event.related_element_ids.forEach(id=>
        document.getElementById(id).classList.add("highlight")
    );
}
function neglet_related_elements(){
    event.related_element_ids.forEach(id=>
        document.getElementById(id).classList.remove("highlight")
    );
}
</script>

<style>
.container {
}
.icon {
}
.name {
}
.desc {
}
</style>

<div class="container" on:mouseover={highlight_related_elements} on:mouseout={neglet_related_elements}>
    <img class="icon" src={event.icon_url} />
    <div class="name"> {event.name} </div>
    <div class="desc"> {event.desc} </div>
</div>
