table! {
    characters (id) {
        id -> Int4,
        firstname -> Text,
        surname -> Nullable<Text>,
        matherid -> Nullable<Int4>,
        fatherid -> Nullable<Int4>,
        ownerid -> Nullable<Int4>,
        jobid -> Nullable<Int4>,
        height -> Float8,
        stats -> Array<Int4>,
        gender -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        image_server_domain -> Nullable<Text>,
        born -> Bool,
    }
}

table! {
    characters_traits (id) {
        id -> Int4,
        characterid -> Nullable<Int4>,
        traitid -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

table! {
    jobs (id) {
        id -> Int4,
        name -> Text,
        description -> Text,
        factors -> Nullable<Array<Int4>>,
    }
}

table! {
    traits (id) {
        id -> Int4,
        name -> Text,
        description -> Text,
        duration -> Nullable<Interval>,
    }
}

table! {
    users (id) {
        id -> Int4,
        userid -> Text,
        password -> Text,
        email -> Text,
        nickname -> Text,
        mana -> Int4,
        max_mana -> Int4,
        mana_charge_per_day -> Int4,
        summon_mana_cost -> Int4,
        mana_updated_at -> Timestamp,
        created_at -> Timestamp,
    }
}

joinable!(characters -> jobs (jobid));
joinable!(characters -> users (ownerid));
joinable!(characters_traits -> characters (characterid));
joinable!(characters_traits -> traits (traitid));

allow_tables_to_appear_in_same_query!(
    characters,
    characters_traits,
    jobs,
    traits,
    users,
);
