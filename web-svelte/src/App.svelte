<script>
import Character from './Character.svelte';
let server_url;
class Application {
    constructor(server_url){
        this.server_url = server_url;
        this.characters = [];
        this.family_tree_root = null;
        this.strangers = [];
        this.events = [];
    }
    parse_characters(characters){
        let root = null;
        let characters_dict = {}
        let strangers = [];
        for(let ch of characters){
            characters_dict[ch.id] = ch;
            ch["children"] = [];
        }
        for(let ch of characters){
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
        for(let ch of characters){
            if(!ch.matherid && !ch.fatherid && !ch.wife && !ch.husband && !(ch.children && ch.children.length))
                strangers.push(ch);
            else
                root = ch;
        }
        while(root.matherid || root.fatherid){
            if(root.matherid)
                root = characters_dict[root.matherid];
            else
                root = characters_dict[root.fatherid];
        }
        this.family_tree_root = root;
        this.strangers = strangers;
    }
    establish_session(){
        return fetch(this.server_url + "/session", {
            method: 'get',
            credentials: 'same-origin',
        })
        .then(res => this.parse_characters(res.json()));
    }
    login(id, password){
        return fetch(this.server_url + "/session", {
            method: 'put',
            credentials: 'same-origin',
            data:JSON.stringify({"id": id, "password": password}),
            headers:{
                'Content-Type': 'application/json'
            }
        });
    }
    logout(){
        return fetch(this.server_url + "/session", {
            method: 'delete',
            credentials: 'same-origin',
            headers:{
                'Content-Type': 'application/json'
            }
        });
    }
    join(id, password, surname, email){
        return fetch(this.server_url + "/users", {
            method: 'post',
            credentials: 'same-origin',
            data:JSON.stringify({
                "id": id,
                "password": password,
                "surname": surname,
                "email": email
            }),
            headers:{
                'Content-Type': 'application/json'
            }
        });
    }
};
let app = new Application(server_url);
app.parse_characters([
    {
    "id": 1,
    "firstname": "개똥",
    "surname": "김",
    "imageurl": "https://www.adorama.com/alc/wp-content/uploads/2016/04/2-4-256x256.jpg",
    "partnerid": 2,
    },
    {
    "id": 2,
    "firstname": "숙자",
    "surname": "말",
    "imageurl": "https://www.nationalgallery.org.uk/server.iip?FIF=/fronts/N-0299-00-000027-WZ-PYR.tif&CNT=1&JTL=2,0",
    },
    {
        "id": 3,
        "firstname": "아무개",
        "surname": "김",
        "matherid": 1,
        "fatherid": 2,
        "imageurl": "https://news.artnet.com/app/news-upload/2018/02/PA_NPG_18_55-Obama-R-1-256x256.jpg",
    },
    {
        "id": 4,
        "firstname": "소똥",
        "surname": "김",
        "matherid": 1,
        "fatherid": 2,
        "imageurl": "https://news.artnet.com/app/news-upload/2016/04/Prince-Warhol--256x256.jpg",
    },
    {
        "id": 8,
        "firstname": "말똥",
        "surname": "김",
        "matherid": 1,
        "fatherid": 2,
        "partnerid": 9,
        "imageurl": "https://www.adorama.com/alc/wp-content/uploads/2016/04/2-4-256x256.jpg",
    },
    {
        "id": 9,
        "firstname": "진",
        "surname": "이",
        "imageurl": "https://news.artnet.com/app/news-upload/2016/04/Prince-Warhol--256x256.jpg",
    },
    {
        "id": 10,
        "firstname": "아무아무개",
        "surname": "김",
        "matherid": 8,
        "fatherid": 9,
        "imageurl": "https://www.nationalgallery.org.uk/server.iip?FIF=/fronts/N-0299-00-000027-WZ-PYR.tif&CNT=1&JTL=2,0",
    },
    {
        "id": 11,
        "firstname": "아무아무개",
        "surname": "송",
        "imageurl": "https://www.nationalgallery.org.uk/server.iip?FIF=/fronts/N-0299-00-000027-WZ-PYR.tif&CNT=1&JTL=2,0",
    },
    {
        "id": 12,
        "firstname": "아무아무개",
        "surname": "최",
        "imageurl": "https://www.nationalgallery.org.uk/server.iip?FIF=/fronts/N-0299-00-000027-WZ-PYR.tif&CNT=1&JTL=2,0",
    }
    ]);
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
