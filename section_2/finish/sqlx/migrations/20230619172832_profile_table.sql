-- Add migration script here
create table profile (
    "id" bigserial primary key,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "user_name" varchar(50) NOT NULL,
    "full_name" varchar(100) NOT NULL
);

insert into profile (user_name, full_name) values ('dave', 'Dave Choi');
insert into profile (user_name, full_name) values ('jon', 'John White');
insert into profile (user_name, full_name) values ('linda', 'Linda Smith');
insert into profile (user_name, full_name) values ('jill', 'Jill Jones');