#!/usr/bin/env bats

load "helpers.bats"

setup() {
    ##
    # On setup, we will initialize a new clean repo under "repo", and then write
    # a simple hierarchy.

    clean_cd "repo"
    ruc init

    echo "a" > a.txt
    echo "b1" > b.txt
    mkdir -p b
    echo "b2" > b/b.txt
}

@test "log detects that no commits have been performed" {
    ruc log

    [ "$output" = "Error: current branch has no commit yet" ]
}

@test "log prints different commits in order" {
    ruc commit -m "First"
    sha1=$(ruc log | head -n 1 | awk '{ print $2; }')

    ruc commit -m "Second"
    sha2=$(ruc log | head -n 1 | awk '{ print $2; }')

    ruc commit -m "Third"
    sha3=$(ruc log | head -n 1 | awk '{ print $2; }')

    ruc log

    [ "${lines[0]}" = "commit ${sha3}" ]
    [ "${lines[1]}" = "Third" ]
    [ "${lines[2]}" = "commit ${sha2}" ]
    [ "${lines[3]}" = "Second" ]
    [ "${lines[4]}" = "commit ${sha1}" ]
    [ "${lines[5]}" = "First" ]
}

@test "log prints different commits from a given object id" {
    ruc commit -m "First"
    sha1=$(ruc log | head -n 1 | awk '{ print $2; }')

    ruc commit -m "Second"
    sha2=$(ruc log | head -n 1 | awk '{ print $2; }')

    ruc commit -m "Third"
    sha3=$(ruc log | head -n 1 | awk '{ print $2; }')

    ruc log --from "${sha2}"
    [ "${lines[0]}" = "commit ${sha2}" ]
    [ "${lines[2]}" = "commit ${sha1}" ]

    ruc log --from "${sha1}"
    [ "${lines[0]}" = "commit ${sha1}" ]

    ruc log --from "${sha3}"
    [ "${lines[0]}" = "commit ${sha3}" ]
    [ "${lines[2]}" = "commit ${sha2}" ]
    [ "${lines[4]}" = "commit ${sha1}" ]
}

@test "checkout works for a given commit id" {
    ruc commit -m "First"
    sha1=$(ruc log | head -n 1 | awk '{ print $2; }')

    echo "b3" > b.txt
    ruc commit -m "Second"
    sha2=$(ruc log | head -n 1 | awk '{ print $2; }')

    [ "$(cat b.txt)" = "b3" ]
    [ "$(cat .ruc/HEAD)" = "${sha2}" ]

    ruc checkout "${sha1}"

    [ "$(cat b.txt)" = "b1" ]
    [ "$(cat .ruc/HEAD)" = "${sha1}" ]
}

@test "tag creates an annotated name for the given commit" {
    ruc commit -m "First"
    sha1=$(ruc log | head -n 1 | awk '{ print $2; }')

    ruc commit -m "Second"
    sha2=$(ruc log | head -n 1 | awk '{ print $2; }')

    ruc tag -a second
    ruc tag -a first "${sha1}"

    [ "$(cat .ruc/HEAD)" = "$(cat .ruc/refs/tags/second)" ]
    [ "${sha1}" = "$(cat .ruc/refs/tags/first)" ]
}
