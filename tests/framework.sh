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
skip.list supports glob patterns: *, ?, and [...] for fine-grained skips.
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
        # shellcheck disable=SC2053
        [[ "$suite" == $skip ]] && return 0
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
        # shellcheck disable=SC2053
        [[ "$key" == $skip ]] && return 0
    done
    return 1
}

is_rom_skipped() {
    local suite="$1"
    local rom_rel_path="$2"
    if [[ ${#SKIP_SUBDIRS[@]} -eq 0 ]]; then
        return 1
    fi
    local full_path="${suite}/${rom_rel_path}"
    for skip in "${SKIP_SUBDIRS[@]}"; do
        # shellcheck disable=SC2053
        [[ "$full_path" == $skip ]] && return 0
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

get_rom_rel_path() {
    local rom_path="$1"
    local suite_dir="$2"
    local roms_dir="${suite_dir}/roms"
    local prefix="${roms_dir}/"
    echo "${rom_path#"$prefix"}"
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
        group_key="${suite_name}"
    else
        group_key="${suite_name}/${group}"
    fi

    GROUP_TOTAL["$group_key"]=${#roms[@]}
    GROUP_FAILED["$group_key"]=0
    GROUP_SKIPPED["$group_key"]=0
    GROUP_RESULTS["$group_key"]=""

    local pids=()
    local pid_roms=()
    local pid_results=()

    for rom in "${roms[@]}"; do
        local rom_name
        rom_name=$(basename "$rom")
        local result_dir
        result_dir=$(mktemp -d "/tmp/xenogb_result_XXXXXX")

        (
            export SUITE_DIR="$suite_dir"
            export SUITE_NAME="$suite_name"
            export ROM_PATH="$rom"
            export OUTPUT_DIR="$result_dir"

            local start_time
            start_time=$(date +%s%N 2>/dev/null || date +%s)

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

            local end_time
            end_time=$(date +%s%N 2>/dev/null || date +%s)
            local elapsed_ns=$(( end_time - start_time ))
            if [[ ${#start_time} -gt 10 ]]; then
                local elapsed_s=$(( elapsed_ns / 1000000000 ))
                local elapsed_frac=$(( (elapsed_ns % 1000000000) / 1000000 ))
                echo "${elapsed_s}.${elapsed_frac}" > "$result_dir/.elapsed"
            else
                echo "$elapsed_ns" > "$result_dir/.elapsed"
            fi
        ) &

        pids+=($!)
        pid_roms+=("$rom_name")
        pid_results+=("$result_dir")
    done

    for i in "${!pids[@]}"; do
        wait "${pids[$i]}" 2>/dev/null || true

        local status_dir="${pid_results[$i]}"
        local rom_name="${pid_roms[$i]}"
        local status="FAIL"
        local elapsed=0

        if [[ -f "$status_dir/.status" ]]; then
            status=$(<"$status_dir/.status")
        fi
        if [[ -f "$status_dir/.elapsed" ]]; then
            elapsed=$(<"$status_dir/.elapsed")
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

record_rom_skipped() {
    local group_key="$1"
    shift
    local roms=("$@")

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
    local suite_name="${group_key%%/*}"
    local subgroup="${group_key#"$suite_name"/}"
    local is_root=0
    [[ "$subgroup" == "$group_key" ]] && is_root=1
    local total=${GROUP_TOTAL["$group_key"]}
    local failed=${GROUP_FAILED["$group_key"]}
    local skipped=${GROUP_SKIPPED["$group_key"]}

    COLLECTED_RESULTS+=("${is_root}|${suite_name}|${subgroup}|${total}|${failed}|${skipped}")
}

COLLECTED_RESULTS=()
SUMMARY_MAX_WIDTH=20

compute_summary_width() {
    local max_w=20
    local suite_name
    for suite_name in "$@"; do
        local suite_dir="$FRAMEWORK_DIR/$suite_name"
        local roms=()
        # shellcheck disable=SC2207
        roms=($(discover_roms "$suite_dir"))
        local rom group rel
        for rom in "${roms[@]}"; do
            group=$(get_rom_group "$rom" "$suite_dir")
            if [[ -z "$group" ]]; then
                rel="$suite_name"
            else
                rel="$group"
            fi
            local len=${#rel}
            (( len > max_w )) && max_w=$len
        done
    done
    SUMMARY_MAX_WIDTH=$max_w
}

print_collected_summaries() {
    if [[ ${#COLLECTED_RESULTS[@]} -eq 0 ]]; then
        return
    fi

    local max_width=$SUMMARY_MAX_WIDTH

    local prev_suite=""
    for entry in "${COLLECTED_RESULTS[@]}"; do
        local is_root suite_name subgroup total failed skipped
        IFS='|' read -r is_root suite_name subgroup total failed skipped <<< "$entry"
        local passed=$(( total - failed - skipped ))

        if [[ "$is_root" == "1" ]]; then
            # Root-level: inline with suite name (2-char indent offset for alignment)
            local padded
            padded=$(printf "%-*s" "$(( max_width + 2 ))" "$suite_name")

            if [[ $passed -eq 0 && $failed -eq 0 ]]; then
                echo -e "${BOLD}${padded}${RESET} \033[0;33m[SKIP]\033[0m ${skipped}/${total}"
            elif [[ $failed -eq 0 ]]; then
                if [[ $skipped -gt 0 ]]; then
                    echo -e "${BOLD}${padded}${RESET} \033[0;32m[PASS]\033[0m ${passed}/${total} (${skipped} skipped)"
                else
                    echo -e "${BOLD}${padded}${RESET} \033[0;32m[PASS]\033[0m ${passed}/${total}"
                fi
            else
                if [[ $skipped -gt 0 ]]; then
                    echo -e "${BOLD}${padded}${RESET} \033[0;31m[FAIL]\033[0m ${passed}/${total} (${failed} failed, ${skipped} skipped)"
                else
                    echo -e "${BOLD}${padded}${RESET} \033[0;31m[FAIL]\033[0m ${passed}/${total} (${failed} failed)"
                fi
            fi
            prev_suite="$suite_name"
        else
            # Non-root: print suite header if new
            if [[ "$suite_name" != "$prev_suite" ]]; then
                echo -e "${BOLD}${suite_name}${RESET}"
                prev_suite="$suite_name"
            fi

            printf "  %-*s" "$max_width" "$subgroup"

            if [[ $passed -eq 0 && $failed -eq 0 ]]; then
                printf " \033[0;33m[SKIP]\033[0m %d/%d\n" "$skipped" "$total"
            elif [[ $failed -eq 0 ]]; then
                if [[ $skipped -gt 0 ]]; then
                    printf " \033[0;32m[PASS]\033[0m %d/%d (%d skipped)\n" "$passed" "$total" "$skipped"
                else
                    printf " \033[0;32m[PASS]\033[0m %d/%d\n" "$passed" "$total"
                fi
            else
                if [[ $skipped -gt 0 ]]; then
                    printf " \033[0;31m[FAIL]\033[0m %d/%d (%d failed, %d skipped)\n" "$passed" "$total" "$failed" "$skipped"
                else
                    printf " \033[0;31m[FAIL]\033[0m %d/%d (%d failed)\n" "$passed" "$total" "$failed"
                fi
            fi
        fi
    done
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
            printf "\033[0;32m[PASS]\033[0m  (%ss)\n" "$elapsed"
        elif [[ "$status" == "SKIP" ]]; then
            printf "\033[0;33m[SKIP]\033[0m\n"
        else
            printf "\033[0;31m[FAIL]\033[0m  (%ss)\n" "$elapsed"
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

    compute_summary_width "${active_suites[@]}"

    for suite_name in "${active_suites[@]}"; do
        local suite_dir="$FRAMEWORK_DIR/$suite_name"

        if is_suite_skipped "$suite_name"; then
            local roms=()
            # shellcheck disable=SC2207
            roms=($(discover_roms "$suite_dir"))
            if [[ ${#roms[@]} -gt 0 ]]; then
                record_group_skipped "${suite_name}" "${roms[@]}"
                if [[ $VERBOSE -eq 1 ]]; then
                    print_group_verbose "${suite_name}"
                else
                    print_group_summary "${suite_name}"
                    print_collected_summaries
                    COLLECTED_RESULTS=()
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

            local display_key="${suite_name}/${group}"
            if [[ -z "$group" ]]; then
                display_key="${suite_name}"
            fi

            if is_subdir_skipped "$suite_name" "$group"; then
                record_group_skipped "$display_key" "${group_rom_array[@]}"
            else
                # Check for individually skipped ROMs via glob patterns
                local run_roms=()
                local skipped_roms=()
                local rom_rel
                for rom in "${group_rom_array[@]}"; do
                    rom_rel=$(get_rom_rel_path "$rom" "$suite_dir")
                    if is_rom_skipped "$suite_name" "$rom_rel"; then
                        skipped_roms+=("$rom")
                    else
                        run_roms+=("$rom")
                    fi
                done

                if [[ ${#run_roms[@]} -gt 0 ]]; then
                    run_group_tests "$suite_name" "$suite_dir" "$group" "${run_roms[@]}"
                fi

                if [[ ${#skipped_roms[@]} -gt 0 ]]; then
                    record_rom_skipped "$display_key" "${skipped_roms[@]}"
                    GROUP_TOTAL["$display_key"]=$(( ${GROUP_TOTAL["$display_key"]:-0} + ${#skipped_roms[@]} ))
                    GROUP_SKIPPED["$display_key"]=$(( ${GROUP_SKIPPED["$display_key"]:-0} + ${#skipped_roms[@]} ))
                fi
            fi

            if [[ $VERBOSE -eq 1 ]]; then
                print_group_verbose "$display_key"
            else
                print_group_summary "$display_key"
            fi
        done

        if [[ $VERBOSE -ne 1 ]]; then
            print_collected_summaries
            COLLECTED_RESULTS=()
        fi

        unset group_roms
    done

    print_total

    if [[ $TOTAL_FAILED -gt 0 ]]; then
        exit 1
    fi
    exit 0
}
