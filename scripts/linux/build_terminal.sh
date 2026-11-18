report_terminal_failure() {
    local message=$1 fd terminal
    printf '\n%s\n' "$message" >&2
    # Read Enter from the visible terminal even when stdin is redirected.
    for fd in 2 1; do
        if [[ -t $fd ]] && exec {terminal}<>"/dev/fd/$fd"; then
            if [[ ! -t 2 ]]; then printf '\n%s\n' "$message" >&"$terminal"; fi
            printf 'Press Enter to close this window... ' >&"$terminal"
            IFS= read -r -u "$terminal" _ || :
            exec {terminal}>&-
            break
        fi
    done
}

# Run the launcher's main function, keeping a distinct log until it succeeds.
run_logged_build() (
    set -euo pipefail
    name=$1 title=$2 log_dir=$3
    shift 3
    log_file='' detail=''

    report_failure() {
        local status=$1 message excerpt=''
        (( status != 0 )) || return 0
        set +e
        message="ERROR: $title failed (exit $status)."
        if [[ -f "$log_file" ]]; then
            message+=" Log: $log_file"
            # Compiler errors can precede many lines of Docker cleanup output.
            excerpt=$(awk 'NF {
                last = $0
                if (count < 3 && tolower($0) ~ /(^|[[:space:]])(error(\[[^]]+\])?:|fatal( error)?:|panicked at|failed to solve:|unknown argument:)/) {
                    print; count++
                }
            } END { if (!count && length(last)) print last }' "$log_file")
        fi
        if [[ -n "$detail" ]]; then message+=$'\n'"$detail"; fi
        if [[ -f "$log_file" ]]; then printf '\n%s\n' "$message" >> "$log_file"; fi
        if [[ -n "$excerpt" ]]; then message+=$'\n'"$excerpt"; fi
        report_terminal_failure "$message"
        exit "$status"
    }
    trap 'report_failure "$?"' EXIT

    if result=$( { mkdir -p -- "$log_dir" && mktemp -- "$log_dir/$name-$(date +%Y%m%d-%H%M%S)-XXXXXX.log"; } 2>&1); then
        log_file=$result
    else
        status=$?
        detail="Could not create the build log: $result"
        log_file=$(mktemp -- "${TMPDIR:-/tmp}/ffta-$name-XXXXXX.log" 2>/dev/null) || :
        exit "$status"
    fi
    printf '%s log: %s\n' "$title" "$log_file"

    # A conditional pipeline would disable errexit inside main. Run it in its
    # own strict subshell and keep the command's exit code ahead of tee's.
    set +e
    (set -e; main "$@") 2>&1 | tee -- "$log_file"
    statuses=("${PIPESTATUS[@]}")
    set -e
    if (( statuses[1] != 0 )); then detail="Could not write the complete build log (tee exit ${statuses[1]})."; fi
    if (( statuses[0] != 0 )); then exit "${statuses[0]}"; fi
    if (( statuses[1] != 0 )); then exit "${statuses[1]}"; fi

    latest="$log_dir/$name-latest.log"
    if result=$(mv -fT -- "$log_file" "$latest" 2>&1); then
        log_file=$latest
        printf 'Build log: %s\n' "$latest"
    else
        status=$?
        detail="Could not publish the success log: $result"
        exit "$status"
    fi
)
