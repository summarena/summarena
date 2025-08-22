#!/bin/sh -eux
test ! -e omni.db
sqlite3 omni.db <schema.sql
