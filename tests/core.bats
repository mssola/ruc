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

@test "can write a tree structure and then read it back" {
    ##
    # 1. Save the current tree with `write-tree`.
    # 2. Nuke everything.
    # 3. Restore things back with `read-tree`.

    ruc write-tree

    rm -f a.txt b.txt
    rm -rf b

    # Finding the tree can be a bit tricky :)
    sha=''
    for file in $(ls .ruc/objects); do
        if [ -n "$(awk '{ if ($1 == "tree" && $3 == "b") print $2 }' .ruc/objects/$file)" ]; then
            sha="$file"
        fi
    done
    [[ -n "$sha" ]]

    ruc read-tree $sha

    [[ -f "a.txt" ]]
    [[ -f "b.txt" ]]
    [[ -f "b/b.txt" ]]
}

@test "cat-file works" {
    ruc write-tree

    # We will find a known object. For this simple case, finding the root tree
    # should cut it.
    sha=''
    for file in $(ls .ruc/objects); do
        if [ -n "$(awk '{ if ($1 == "tree" && $3 == "b") print $2 }' .ruc/objects/$file)" ]; then
            sha="$file"
        fi
    done
    [[ -n "$sha" ]]

    ##
    # The root tree is guaranteed to have 5 entries (2 lines from presentation
    # and 3 actual entries). From these, 2 are blobs.

    lines=$(ruc cat-file $sha | wc -l)
    [[ "$lines" = "5" ]]

    lines=$(ruc cat-file $sha | grep blob | wc -l)
    [[ "$lines" = "2" ]]
}
