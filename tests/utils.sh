#!/usr/bin/env bash

cmp_regs () {
    local cpu_out
    cpu_out=$(<"$1")
    shift

    for reg; do
        [[ $cpu_out =~ $reg ]] || return 1
    done
    return 0
}

cmp_snapshot () {
    convert $1.pgm $1.png
    compare -metric AE "$1.png" "$2" NULL: 2>&1 > /dev/null
    [ $? -eq 0 ]
}
