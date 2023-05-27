#!/usr/bin/env bats

ruc() {
    run cargo -q run -- "$@"
}

clean_cd() {
    DIR="$( cd "$( dirname "$BATS_TEST_FILENAME" )" >/dev/null 2>&1 && pwd )"

    mkdir -p "${DIR}/${1}"
    rm -rf "${DIR}/${1}/.ruc"
    cd "${DIR}/${1}" || exit 1
}
