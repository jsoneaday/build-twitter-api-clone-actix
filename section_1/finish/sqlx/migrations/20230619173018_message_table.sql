-- Add migration script here
create table message (
    "id" bigserial primary key,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "user_id" bigserial NOT NULL,
    "body"  varchar(140),
    "likes" int NOT NULL DEFAULT 0,

    constraint fk_profile foreign key(user_id) references profile(id)
);

insert into message (user_id, body, likes) values (1, 'Hello Guys!', 5);
insert into message (user_id, body, likes) values (2, 'Hello Girls!', 2);