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
    if command -v magick &>/dev/null; then
        magick "$1".ppm "$1".png
    else
        convert "$1".ppm "$1".png
    fi
    compare -metric AE "$1".png "$2" NULL: 2>/dev/null
}
