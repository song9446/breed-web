<script context="module">
</script>
<script>
export let event = null;
function highlight_related_elements(){
    console.log("hovered..");
    event.related_element_ids.forEach(id=>
        console.log("highlight..", id);
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
    border-bottom: 1px dotted black;
}
.icon {
}
.name {
    overflow: hidden;
}
.container .desc {
    visibility: hidden;
    position: absolute;
    z-index: 1;
    padding: 5px 0;
    background-color: black;
    margin-left: -5px;
    border-width: 5px;
    border-style: solid;
    border-color: transparent transparent black transparent;
    border-radius: 6px;
    top: -5px;
    left: 110%;
}
.container:hover .desc {
    visibility: visible;
}
.container .desc::after {
    content: "";
    position: absolute;
    top: 0.8rem;
    right: 100%;
    margin-top: -5px;
    border-width: 5px;
    border-style: solid;
    border-color: transparent black transparent transparent;
    visibility: visible;
}
</style>

<div class="container" on:mouseover={highlight_related_elements} on:mouseout={neglet_related_elements}>
    <img class="icon" src={event.icon_url} />
    <div class="name"> {event.name} </div>
    <div class="desc"> {event.desc} </div>
</div>
