#!/bin/bash

# this is a list of various migration related commands
# make sure you've installed sqlx-cli and postgres
# uncomment the one you want to run

# test psql on command line using: psql -h localhost -p 5432 -d tester -U tester

# sqlx database create --database-url postgres://tester:tester@localhost:5432/tester
# sqlx migrate add profile_table
# sqlx migrate add message_table
# sqlx migrate run --database-url postgres://tester:tester@localhost:5432/tester