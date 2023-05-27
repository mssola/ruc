#!/usr/bin/env bats

load "helpers.bats"

setup() {
    clean_cd "repo"
}

@test "init: initializes the repository" {
    ruc init

    [[ -d ".ruc" ]]
    [[ -d ".ruc/objects" ]]
}

@test "init: leaves a repo alone when already created" {
    ruc init
    echo "hello" >> ".ruc/objects/lala.txt"

    ruc init

    [[ -d ".ruc" ]]
    [[ -f ".ruc/objects/lala.txt" ]]
}
