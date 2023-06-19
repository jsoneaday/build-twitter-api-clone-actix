#!/bin/bash

# this is a list of various migration related commands
# uncomment the one you want to run

# test psql on command line using: psql -h localhost -p 5432 -d tester -U tester

# cargo sqlx database create --database-url postgres://tester:tester@localhost:5432/tester
# cargo sqlx migrate add profile_table
# cargo sqlx migrate add message_table
cargo sqlx migrate run --database-url postgres://tester:tester@localhost:5432/tester