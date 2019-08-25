class App {
    constructor(server_url) {
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
            if(ch.matherid)
                characters_dict[ch.matherid]["children"].push(ch);
            if(ch.partnerid)
                ch["partner"] = characters_dict[ch.partnerid];
        }
        for(let ch of characters){
            if(!ch.matherid && !ch.fatherid && !ch.partnerid && !ch.children)
                strangers.push(ch);
            else
                root = ch;
        }
        while(root.matherid or root.fatherid){
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
