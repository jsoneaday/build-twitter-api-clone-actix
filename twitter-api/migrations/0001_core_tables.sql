create table profile (
    "id" bigserial primary key,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "user_name" varchar(50) NOT NULL,
    "full_name" varchar(100) NOT NULL,
    "description" varchar(250) NOT NULL,
    "region" varchar(50),
    "main_url" varchar(250),
    "avatar" bytea
);

create table follow (
    "id" bigserial primary key,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "follower_id" bigserial NOT NULL,
    "following_id" bigserial NOT NULL,

    constraint fk_profile_follower foreign key(follower_id) references profile(id),
    constraint fk_profile_following foreign key(following_id) references profile(id)
);

create table message (
    "id" bigserial primary key,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "user_id" bigserial NOT NULL,
    "body"  varchar(140),
    "likes" int NOT NULL DEFAULT 0,
    "image" bytea,
    "msg_group_type" int,

    constraint fk_profile foreign key(user_id) references profile(id)
);

create table message_response (
    "id" bigserial primary key,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "original_msg_id" bigserial NOT NULL,
    "responding_msg_id" bigserial NOT NULL,

    constraint fk_original_message foreign key(original_msg_id) references message(id),
    constraint fk_responding_message foreign key(responding_msg_id) references message(id)
);

create table message_broadcast (
    "id" bigserial primary key,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "main_msg_id" bigserial NOT NULL,
    "broadcasting_msg_id" bigserial NOT NULL,

    constraint fk_original_message foreign key(main_msg_id) references message(id),
    constraint fk_broadcasting_message foreign key(broadcasting_msg_id) references message(id)
);
