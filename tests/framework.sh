#!/usr/bin/env bash
# framework.sh — Core test framework for XenoGB
# Sourced by run_tests.sh. Do not execute directly.

# Note: do NOT use 'set -euo pipefail' here — this file is sourced,
# and re-applying set -u breaks array handling with existing vars.

# ─── Colors ───────────────────────────────────────────────────────────────────

BOLD='\033[1m'
RESET='\033[0m'

# ─── Globals (preserve values set by run_tests.sh) ────────────────────────────

FRAMEWORK_DIR="${FRAMEWORK_DIR:-}"
EXEC="${EXEC:-}"
VERBOSE="${VERBOSE:-0}"

declare -A GROUP_TOTAL
declare -A GROUP_FAILED
declare -A GROUP_SKIPPED
declare -A GROUP_RESULTS

TOTAL_TESTS=0
TOTAL_PASSED=0
TOTAL_FAILED=0
TOTAL_SKIPPED=0

# ─── CLI Parsing ──────────────────────────────────────────────────────────────

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help)
                print_usage
                exit 0
                ;;
            -v|--verbose)
                VERBOSE=1
                shift
                ;;
            -s|--suite)
                SUITE_FILTERS+=("$2")
                shift 2
                ;;
            --skip-suite)
                SKIP_SUITES+=("$2")
                shift 2
                ;;
            --skip-subdir)
                SKIP_SUBDIRS+=("$2")
                shift 2
                ;;
            *)
                echo "Unknown option: $1" >&2
                print_usage
                exit 1
                ;;
        esac
    done
}

print_usage() {
    cat <<'EOF'
Usage: ./tests/run_tests.sh [OPTIONS]

Options:
  -h, --help            Print this help and exit
  -v, --verbose         Verbose output (per-ROM results with timing)
  -s, --suite <name>    Run only the named suite(s) (repeatable)
      --skip-suite <name>     Skip the named suite(s) (repeatable)
      --skip-subdir <path>    Skip subdirectory (e.g. mooneye/timer) (repeatable)

Exclusion always wins over inclusion. skip.list entries are merged with CLI skips.
EOF
}

# ─── Skip List ────────────────────────────────────────────────────────────────

load_skip_list() {
    local skip_file="$FRAMEWORK_DIR/skip.list"
    [[ -f "$skip_file" ]] || return 0

    while IFS= read -r line || [[ -n "$line" ]]; do
        line="${line%%#*}"
        line="${line// /}"
        [[ -z "$line" ]] && continue

        if [[ "$line" == */* ]]; then
            SKIP_SUBDIRS+=("$line")
        else
            SKIP_SUITES+=("$line")
        fi
    done < "$skip_file"
}

# ─── Discovery ────────────────────────────────────────────────────────────────

discover_suites() {
    local suites=()
    for dir in "$FRAMEWORK_DIR"/*/; do
        [[ -d "$dir" ]] || continue
        local name
        name=$(basename "$dir")
        [[ -f "$dir/config.sh" ]] || continue
        suites+=("$name")
    done
    echo "${suites[@]}"
}

is_suite_skipped() {
    local suite="$1"
    if [[ ${#SKIP_SUITES[@]} -eq 0 ]]; then
        return 1
    fi
    for skip in "${SKIP_SUITES[@]}"; do
        [[ "$skip" == "$suite" ]] && return 0
    done
    return 1
}

is_subdir_skipped() {
    local suite="$1"
    local subdir="$2"
    if [[ ${#SKIP_SUBDIRS[@]} -eq 0 ]]; then
        return 1
    fi
    local key="${suite}/${subdir}"
    for skip in "${SKIP_SUBDIRS[@]}"; do
        [[ "$skip" == "$key" ]] && return 0
    done
    return 1
}

is_suite_filtered() {
    if [[ ${#SUITE_FILTERS[@]} -eq 0 ]]; then
        return 0
    fi
    for filter in "${SUITE_FILTERS[@]}"; do
        [[ "$filter" == "$1" ]] && return 0
    done
    return 1
}

discover_roms() {
    local suite_dir="$1"
    local roms_dir="${suite_dir}/roms"
    [[ -d "$roms_dir" ]] || return 0

    find "$roms_dir" -type f \( -name '*.gb' -o -name '*.gbc' \) | sort
}

get_rom_group() {
    local rom_path="$1"
    local suite_dir="$2"
    local roms_dir="${suite_dir}/roms"
    local prefix="${roms_dir}/"
    local rel_path="${rom_path#"$prefix"}"
    local dir
    dir=$(dirname "$rel_path")
    [[ "$dir" == "." ]] && echo "" || echo "$dir"
}

# ─── Execution ────────────────────────────────────────────────────────────────

run_group_tests() {
    local suite_name="$1"
    local suite_dir="$2"
    local group="$3"
    shift 3
    local roms=("$@")

    local group_key
    if [[ -z "$group" ]]; then
        group_key="${suite_name}/roms"
    else
        group_key="${suite_name}/roms/${group}"
    fi

    GROUP_TOTAL["$group_key"]=${#roms[@]}
    GROUP_FAILED["$group_key"]=0
    GROUP_SKIPPED["$group_key"]=0
    GROUP_RESULTS["$group_key"]=""

    local pids=()
    local pid_roms=()
    local pid_results=()
    local pid_starts=()

    for rom in "${roms[@]}"; do
        local rom_name
        rom_name=$(basename "$rom")
        local result_dir
        result_dir=$(mktemp -d "/tmp/xenogb_result_XXXXXX")
        local start_time
        start_time=$(date +%s)

        (
            export SUITE_DIR="$suite_dir"
            export SUITE_NAME="$suite_name"
            export ROM_PATH="$rom"
            export OUTPUT_DIR="$result_dir"

            local run_exit=0
            run_test "$rom" "$result_dir" || run_exit=$?

            if [[ $run_exit -ne 0 ]]; then
                echo "FAIL" > "$result_dir/.status"
            else
                local check_exit=0
                check_test "$rom" "$result_dir" || check_exit=$?
                if [[ $check_exit -ne 0 ]]; then
                    echo "FAIL" > "$result_dir/.status"
                else
                    echo "PASS" > "$result_dir/.status"
                fi
            fi
        ) &

        pids+=($!)
        pid_roms+=("$rom_name")
        pid_results+=("$result_dir")
        pid_starts+=("$start_time")
    done

    for i in "${!pids[@]}"; do
        wait "${pids[$i]}" 2>/dev/null || true
        local end_time
        end_time=$(date +%s)
        local elapsed=$(( end_time - ${pid_starts[$i]} ))

        local status_dir="${pid_results[$i]}"
        local rom_name="${pid_roms[$i]}"
        local status="FAIL"

        if [[ -f "$status_dir/.status" ]]; then
            status=$(<"$status_dir/.status")
        fi

        rm -rf "$status_dir"

        GROUP_RESULTS["$group_key"]+="${rom_name}:${status}:${elapsed} "
        TOTAL_TESTS=$((TOTAL_TESTS + 1))

        if [[ "$status" == "PASS" ]]; then
            TOTAL_PASSED=$((TOTAL_PASSED + 1))
        else
            TOTAL_FAILED=$((TOTAL_FAILED + 1))
            GROUP_FAILED["$group_key"]=$(( ${GROUP_FAILED["$group_key"]} + 1 ))
        fi
    done
}

record_group_skipped() {
    local group_key="$1"
    shift
    local roms=("$@")

    GROUP_TOTAL["$group_key"]=${#roms[@]}
    GROUP_FAILED["$group_key"]=0
    GROUP_SKIPPED["$group_key"]=${#roms[@]}
    GROUP_RESULTS["$group_key"]=""

    local rom
    for rom in "${roms[@]}"; do
        local rom_name
        rom_name=$(basename "$rom")
        GROUP_RESULTS["$group_key"]+="${rom_name}:SKIP:0 "
        TOTAL_SKIPPED=$((TOTAL_SKIPPED + 1))
    done
}

# ─── Output ───────────────────────────────────────────────────────────────────

declare -g PRINTED_SUITE=""

print_group_summary() {
    local group_key="$1"
    local suite_name="${group_key%%/roms*}"
    local prefix="${suite_name}/"
    local subgroup="${group_key#"$prefix"}"
    local total=${GROUP_TOTAL["$group_key"]}
    local failed=${GROUP_FAILED["$group_key"]}
    local skipped=${GROUP_SKIPPED["$group_key"]}

    if [[ "$suite_name" != "$PRINTED_SUITE" ]]; then
        echo -e "${BOLD}${suite_name}${RESET}"
        PRINTED_SUITE="$suite_name"
    fi

    printf "  %-22s" "$subgroup"

    if [[ $skipped -gt 0 ]]; then
        printf " \033[0;33m[SKIP]\033[0m %d/%d\n" "$skipped" "$total"
    elif [[ $failed -eq 0 ]]; then
        printf " \033[0;32m[PASS]\033[0m %d/%d\n" "$total" "$total"
    else
        printf " \033[0;31m[FAIL]\033[0m %d/%d (%d failed)\n" "$(( total - failed ))" "$total" "$failed"
    fi
}

print_group_verbose() {
    local group_key="$1"
    local results="${GROUP_RESULTS["$group_key"]}"

    if [[ "$group_key" != "$PRINTED_SUITE" ]]; then
        echo -e "${BOLD}${group_key}${RESET}"
        PRINTED_SUITE="$group_key"
    fi

    local entry
    for entry in $results; do
        local name status elapsed
        name="${entry%%:*}"
        local rest="${entry#*:}"
        status="${rest%%:*}"
        elapsed="${rest#*:}"

        printf "  %-35s" "$name"
        if [[ "$status" == "PASS" ]]; then
            printf "\033[0;32m[PASS]\033[0m  (%ds)\n" "$elapsed"
        elif [[ "$status" == "SKIP" ]]; then
            printf "\033[0;33m[SKIP]\033[0m\n"
        else
            printf "\033[0;31m[FAIL]\033[0m  (%ds)\n" "$elapsed"
        fi
    done
    echo
}

print_total() {
    echo "──────────────────────────────────────"
    if [[ $TOTAL_SKIPPED -gt 0 ]]; then
        echo "Total:  $TOTAL_TESTS tests — $TOTAL_PASSED passed, $TOTAL_FAILED failed, $TOTAL_SKIPPED skipped"
    else
        echo "Total:  $TOTAL_TESTS tests — $TOTAL_PASSED passed, $TOTAL_FAILED failed"
    fi
}

# ─── Main ─────────────────────────────────────────────────────────────────────

run_framework() {
    local all_suites=()
    # shellcheck disable=SC2207
    all_suites=($(discover_suites))

    if [[ ${#all_suites[@]} -eq 0 ]]; then
        echo "No test suites found." >&2
        exit 1
    fi

    local active_suites=()

    for suite_name in "${all_suites[@]}"; do
        if ! is_suite_filtered "$suite_name"; then
            continue
        fi
        active_suites+=("$suite_name")
    done

    if [[ ${#active_suites[@]} -eq 0 ]]; then
        echo "No suites to run (all filtered or skipped)." >&2
        exit 0
    fi

    for suite_name in "${active_suites[@]}"; do
        local suite_dir="$FRAMEWORK_DIR/$suite_name"

        if is_suite_skipped "$suite_name"; then
            local roms=()
            # shellcheck disable=SC2207
            roms=($(discover_roms "$suite_dir"))
            if [[ ${#roms[@]} -gt 0 ]]; then
                record_group_skipped "${suite_name}/roms" "${roms[@]}"
                if [[ $VERBOSE -eq 1 ]]; then
                    print_group_verbose "${suite_name}/roms"
                else
                    print_group_summary "${suite_name}/roms"
                fi
            fi
            continue
        fi

        # shellcheck disable=SC1091
        source "$suite_dir/config.sh"

        local roms=()
        # shellcheck disable=SC2207
        roms=($(discover_roms "$suite_dir"))

        if [[ ${#roms[@]} -eq 0 ]]; then
            continue
        fi

        declare -A group_roms

        for rom in "${roms[@]}"; do
            local group
            group=$(get_rom_group "$rom" "$suite_dir")
            local group_key="${group:-__root__}"

            if [[ -z "${group_roms[$group_key]+x}" ]]; then
                group_roms["$group_key"]="$rom"
            else
                group_roms["$group_key"]+="|$rom"
            fi
        done

        for group_key in "${!group_roms[@]}"; do
            local group="${group_key#__root__}"
            local old_ifs="$IFS"
            IFS='|'
            # shellcheck disable=SC2206
            local group_rom_array=(${group_roms["$group_key"]})
            IFS="$old_ifs"

            local display_key="${suite_name}/roms/${group}"
            if [[ -z "$group" ]]; then
                display_key="${suite_name}/roms"
            fi

            if is_subdir_skipped "$suite_name" "$group"; then
                record_group_skipped "$display_key" "${group_rom_array[@]}"
            else
                run_group_tests "$suite_name" "$suite_dir" "$group" "${group_rom_array[@]}"
            fi

            if [[ $VERBOSE -eq 1 ]]; then
                print_group_verbose "$display_key"
            else
                print_group_summary "$display_key"
            fi
        done

        unset group_roms
    done

    print_total

    if [[ $TOTAL_FAILED -gt 0 ]]; then
        exit 1
    fi
    exit 0
}
