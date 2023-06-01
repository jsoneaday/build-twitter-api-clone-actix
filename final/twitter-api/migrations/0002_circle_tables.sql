create table circle_group (
    "id" bigserial primary key,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,    
    "owner_id" bigserial NOT NULL,

    constraint fk_profile foreign key(owner_id) references profile(id)
);

create table circle_group_member (
    "id" bigserial primary key,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "circle_group_id" bigserial NOT NULL,
    "member_id" bigserial NOT NULL,

    constraint fk_circle_group foreign key(circle_group_id) references circle_group(id),
    constraint fk_profile foreign key(member_id) references profile(id)
);