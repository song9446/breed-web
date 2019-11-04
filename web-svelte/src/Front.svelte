
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
    <Character character={app.family_tree_root} />
</section>
<section class="strangers">
    <h1>
        Strangers
    </h1>
    <ul>
        {#each app.strangers as stranger}
        <li>
            <Character character={stranger} />
        </li>
        {/each}
    </ul>
</section>
</div>
