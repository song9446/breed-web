export class Application {
    constructor(server_url, image_server_url){
        this.server_url = server_url;
        this.image_server_url = image_server_url;
    }
    gamedata(){
        return this._gamedata;
    }
    character_image_url(ch){
        return this.image_server_url + "/" + ch.id + ".png"
    }
	update(){
        return fetch(this.server_url + "/events", {
            method: 'get',
            credentials: 'include',
		})
        .then(res=>res.json())
	}
    reload_session(){
        return fetch(this.server_url + "/session", {
            method: 'get',
            credentials: 'include',
            headers:{
                'Content-Type': 'application/json'
            }
        })
        .then(res=>res.json())
    }
    login(id, password){
        return fetch(this.server_url + "/session", {
            method: 'post',
            credentials: 'include',
            body:JSON.stringify({"id": id, "password": password}),
            headers:{
                'Content-Type': 'application/json'
            }
        })
        .then(res=>res.json())
    }
    logout(){
        return fetch(this.server_url + "/session", {
            method: 'delete',
            credentials: 'include',
            headers:{
                'Content-Type': 'application/json'
            }
        })
        .then(res=>res.json())
    }
    join(id, password, nickname, email){
        return fetch(this.server_url + "/accounts", {
            method: 'post',
            credentials: 'include',
            body:JSON.stringify({
                "id": id,
                "password": password,
                "nickname": nickname,
                "email": email
            }),
            headers:{
                'Content-Type': 'application/json'
            }
        })
        .then(res=>res.json());
    }
    summon_character(user) {
        if(this.update_mana(user).mana - user.summon_mana_cost < 0)
            return Promise.resolve({"error": {code:401, message:"Not enough mana"}});
        return fetch(this.server_url + "/characters", {
            method: 'post',
            credentials: 'include',
            headers:{
                'Content-Type': 'application/json'
            }
        })
        .then(res=>res.json())
    }
    marry(ch1, ch2) {
        if(ch1.owner != ch2.owner || ch1.partnerid != null || ch2.partnerid == null)
            return Promise.resolve({"error": {code:401, message:"Characters already have a partner"}});
        return fetch(this.server_url + "/marriage", {
            method: 'post',
            body:JSON.stringify({"marrier": [ch1.id, ch2.id]}),
            credentials: 'include',
            headers:{
                'Content-Type': 'application/json'
            }
        })
        .then(res=>res.json())
    }
    update_mana(user) {
        let now = new Date();
        let charged_mana =
            user.mana_charge_per_day *
            ((now.getTime() - (new Date(user.mana_updated_at + (user.mana_updated_at.endsWith("Z")? "": "Z"))).getTime()) / (1000*3600*24));
        let new_mana = Math.min(user.mana + charged_mana, user.max_mana);
        return {"mana": new_mana, "mana_updated_at": now.toISOString()};
    }
    dummy_gamedata(){
        return {
            user:{
                nickname:"dummy"
            },
            characters:[
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
        ],
    };
    }
};
export const app = new Application("http://127.0.0.1:3000", "http://127.0.0.1:8000");
