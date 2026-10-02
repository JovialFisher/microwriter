#!/usr/bin/env bash
#
# microwriter launcher -- macOS and Linux.
#
# Double-click this file (or run it in a terminal) to start microwriter.
#
# If microwriter has not been built on this computer yet, the launcher builds
# it once with all the compiler output redirected to target/build.log, and
# draws a friendly progress line instead (spinner, percentage, bar, elapsed).
# Nothing here should frighten someone who has never used a terminal.
#
# A build that was interrupted (the window closed, Ctrl+C) or that failed
# leaves a broken half-built folder behind. The launcher detects that, deletes
# the broken folder and starts over in a fresh run, so a stopped setup never
# turns into a broken microwriter.

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
cd "$ROOT_DIR" || exit 1

# Absolute path to this script, so it can restart itself even from a relative
# argument (the working directory has just changed above).
SELF="$ROOT_DIR/$(basename -- "$0")"

RELEASE_BIN="target/release/microwriter"
DEBUG_BIN="target/debug/microwriter"
BUILD_LOG="target/build.log"

# A completed build writes this marker beside the app. Its absence means the
# last build was interrupted or failed, so the app cannot be trusted.
BUILD_MARKER="target/build.complete"

# A flag in the temp dir records that the launcher has already restarted once,
# so a genuinely broken build cannot loop forever.
REBUILD_FLAG="${TMPDIR:-/tmp}/microwriter-rebuild.flag"

# Keep the original arguments so the script can restart itself with them.
SCRIPT_ARGS=("$@")

# Width of the progress bar, in columns.
BAR_WIDTH=24

say() { printf '%s\n' "$*"; }

# Delete the broken build folder and start over. On Windows the launcher closes
# its window and opens a fresh one; here the portable equivalent is to throw the
# folder away and replace this script with a fresh run of itself. A macOS
# `.command` gets a brand new Terminal window.
restart_fresh() {
    say ""
    say "This setup did not finish. Clearing the broken build folder and"
    say "starting over..."
    say ""
    rm -rf target
    : > "$REBUILD_FLAG"
    case "$SELF" in
        *.command)
            if command -v open >/dev/null 2>&1; then
                open -- "$SELF"
                exit 0
            fi
            ;;
    esac
    exec "$SELF" "${SCRIPT_ARGS[@]}"
}

# Keep the window open when the launcher was started by double-clicking, so a
# message doesn't vanish before it can be read. Skipped when output is piped.
pause_before_close() {
    [ -t 0 ] || return 0
    printf '\n%s' "Press Enter to close this window..."
    read -r _ || true
}

# Cargo is normally on PATH, but a double-clicked launcher may not inherit
# ~/.cargo/bin, so look there before giving up.
find_cargo() {
    if command -v cargo >/dev/null 2>&1; then
        command -v cargo
        return 0
    fi
    if [ -x "$HOME/.cargo/bin/cargo" ]; then
        printf '%s\n' "$HOME/.cargo/bin/cargo"
        return 0
    fi
    return 1
}

# ---------------------------------------------------------------------------
# Progress display
# ---------------------------------------------------------------------------

# Braille spinners and block bars need a UTF-8 terminal; anything else falls
# back to plain ASCII so a fancy glyph never shows up as a question mark.
unicode=0
case "${LC_ALL:-${LC_CTYPE:-${LANG:-}}}" in
    '' | *[Uu][Tt][Ff]-8* | *[Uu][Tt][Ff]8*)
        unicode=1
        ;;
esac
if [ "$unicode" -eq 1 ]; then
    FRAMES=(⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏)
    BAR_FILLED='█'
    BAR_EMPTY='░'
else
    FRAMES=('|' '/' '-' '\')
    BAR_FILLED='#'
    BAR_EMPTY='-'
fi

# `expected` is how many crates cargo is going to build, or 0 when it could not
# be counted (the progress line then shows the running count instead of a
# percentage). Cargo's own "Compiling ..." lines give the number already done.
expected=0

# How many crates `cargo metadata` lists for this machine. Filtering to the
# local target keeps the number close to what cargo actually compiles.
count_expected_crates() {
    local rustc_bin="" host="" metadata=""
    if command -v rustc >/dev/null 2>&1; then
        rustc_bin="$(command -v rustc)"
    elif [ -x "${CARGO%/*}/rustc" ]; then
        rustc_bin="${CARGO%/*}/rustc"
    fi
    if [ -n "$rustc_bin" ]; then
        host="$("$rustc_bin" -vV 2>/dev/null | sed -n 's/^host: //p')"
    fi
    if [ -n "$host" ]; then
        metadata="$("$CARGO" metadata --format-version 1 --filter-platform "$host" 2>/dev/null)"
    else
        metadata="$("$CARGO" metadata --format-version 1 2>/dev/null)"
    fi
    # cargo metadata prints one long JSON line, so count matches rather than
    # lines: one "manifest_path" per package.
    printf '%s' "$metadata" | grep -o '"manifest_path"' 2>/dev/null | wc -l
}

# One line of progress, e.g.
#   ⠹  Building      42%  [██████████░░░░░░░░░░░░░░]  0:48
progress_line() {
    local done="$1" elapsed="$2" spin="$3"
    local mmss pct filled empty bar i dots label

    mmss="$(printf '%d:%02d' "$((elapsed / 60))" "$((elapsed % 60))")"

    if [ "$done" -le 0 ]; then
        # Nothing compiled yet: cargo is starting up or fetching dependencies.
        dots=""
        i=0
        while [ "$i" -lt $((elapsed % 4 + 1)) ]; do
            dots="$dots."
            i=$((i + 1))
        done
        if [ "$elapsed" -lt 12 ]; then
            label="Getting started"
        else
            label="Fetching what it needs"
        fi
        printf '%s  %s%s  %s' "$spin" "$label" "$dots" "$mmss"
        return
    fi

    if [ "$expected" -le 0 ]; then
        printf '%s  %-13s %s pieces done  %s' "$spin" "Building" "$done" "$mmss"
        return
    fi

    pct=$((done * 100 / expected))
    [ "$pct" -gt 99 ] && pct=99
    # The last crate is microwriter itself, and its linking step takes a while,
    # so say what is happening rather than looking stuck at 99%.
    if [ "$done" -ge $((expected - 1)) ]; then
        label="Finishing up"
    else
        label="Building"
    fi
    filled=$((pct * BAR_WIDTH / 100))
    empty=$((BAR_WIDTH - filled))
    bar=""
    i=0
    while [ "$i" -lt "$filled" ]; do
        bar="${bar}${BAR_FILLED}"
        i=$((i + 1))
    done
    i=0
    while [ "$i" -lt "$empty" ]; do
        bar="${bar}${BAR_EMPTY}"
        i=$((i + 1))
    done
    printf '%s  %-13s %2d%%  [%s]  %s' "$spin" "$label" "$pct" "$bar" "$mmss"
}

# ---------------------------------------------------------------------------
# 0. Did the last build finish, and has this setup already restarted once?
#    A finished build leaves a marker beside the app. If the marker is missing
#    but build output is sitting there, the last setup was interrupted or
#    failed, so the folder may be half-written and broken: delete it and start
#    again. One restart only, so a genuine error cannot loop forever.
# ---------------------------------------------------------------------------
restarted=0
if [ -e "$REBUILD_FLAG" ]; then
    restarted=1
    rm -f "$REBUILD_FLAG"
fi

broken=0
if [ -e "$BUILD_MARKER" ]; then
    :
elif [ -d target/release ] || [ -d target/debug ] || [ -e "$RELEASE_BIN" ] || [ -e "$DEBUG_BIN" ]; then
    broken=1
fi
if [ "$broken" -eq 1 ] && [ "$restarted" -eq 0 ]; then
    restart_fresh
fi

# ---------------------------------------------------------------------------
# 1. Already built? Start immediately -- no build, no waiting.
# ---------------------------------------------------------------------------
if [ -x "$RELEASE_BIN" ] && [ -e "$BUILD_MARKER" ]; then
    exec "$RELEASE_BIN" "${SCRIPT_ARGS[@]}"
fi

if [ -x "$DEBUG_BIN" ] && [ -e "$BUILD_MARKER" ]; then
    exec "$DEBUG_BIN" "${SCRIPT_ARGS[@]}"
fi

# ---------------------------------------------------------------------------
# 2. First run on this computer: build once, quietly.
# ---------------------------------------------------------------------------
say "microwriter"
say "distraction-free writing environment"
say ""
if [ "$restarted" -eq 1 ]; then
    say "Starting over with a clean build folder."
else
    say "It looks like this is the first time you have opened microwriter here."
    say "Let me set everything up for you. This only happens once."
fi
say ""

if ! CARGO="$(find_cargo)"; then
    say "microwriter needs a one-time setup with a free tool called Rust."
    say ""
    say "  1. Visit https://rustup.rs"
    say "  2. Follow the short instructions there, accepting the defaults."
    say "  3. Open microwriter again - everything else happens automatically."
    say ""
    pause_before_close
    exit 1
fi

mkdir -p target

say "Getting ready now. This usually takes one to three minutes."
say "Please leave this window open. Writing will start on its own when done."
say ""

expected="$(count_expected_crates)"
case "$expected" in '' | *[!0-9]*) expected=0 ;; esac

# Width used to pad (and so clear) the progress line, so it never wraps onto a
# second row in a narrow terminal.
line_width=72
if [ -t 1 ]; then
    columns="$(tput cols 2>/dev/null || printf '')"
    case "$columns" in
        '' | *[!0-9]*) : ;;
        *) [ "$columns" -gt 8 ] && line_width=$((columns - 1)) ;;
    esac
fi

build_pid=""
clear_progress() {
    [ "$animated" -eq 1 ] || return 0
    printf "\r%${line_width}s\r" ''
}
cleanup() {
    if [ -n "$build_pid" ] && kill -0 "$build_pid" 2>/dev/null; then
        kill "$build_pid" 2>/dev/null
        wait "$build_pid" 2>/dev/null
    fi
    rm -f "$BUILD_MARKER" "$RELEASE_BIN" "$DEBUG_BIN"
    clear_progress
    say ""
    say "Setup stopped. The unfinished build will be cleared out;"
    say "open microwriter again whenever you like to start over."
    exit 130
}
animated=0
trap cleanup INT TERM

# ---------------------------------------------------------------------------
# 3. Build, with a quiet progress line instead of compiler output. If a build
#    is interrupted or fails, the broken folder is thrown away and the whole
#    setup starts over fresh rather than asking the user to do it.
# ---------------------------------------------------------------------------

# Start from a clean slate: withdraw the marker and clear the app's own
# artifacts, so whatever a stopped build left behind cannot get in the way.
rm -f "$BUILD_MARKER"
"$CARGO" clean -p microwriter >/dev/null 2>&1 || true

SECONDS=0
"$CARGO" build --release >"$BUILD_LOG" 2>&1 &
build_pid=$!

# The line rewrites itself in place in a terminal; piped output gets one plain
# progress line per milestone instead, so a log stays readable.
[ -t 1 ] && animated=1

frame=0
milestone=-1
while kill -0 "$build_pid" 2>/dev/null; do
    done_crates=0
    if [ -f "$BUILD_LOG" ]; then
        done_crates="$(grep -c 'Compiling' "$BUILD_LOG" 2>/dev/null)"
        case "$done_crates" in '' | *[!0-9]*) done_crates=0 ;; esac
    fi

    line="$(progress_line "$done_crates" "$SECONDS" "${FRAMES[$((frame % ${#FRAMES[@]}))]}")"
    if [ "$animated" -eq 1 ]; then
        printf "\r%${line_width}s" "$line"
    else
        if [ "$done_crates" -gt 0 ] && [ "$expected" -gt 0 ]; then
            # One line per 20% of the build.
            step=$((done_crates * 100 / expected / 20))
        else
            # Still warming up or fetching: one line every 15 seconds.
            step=$((SECONDS / 15))
        fi
        if [ "$step" -ne "$milestone" ]; then
            milestone="$step"
            say "$line"
        fi
    fi

    frame=$((frame + 1))
    sleep 0.25
done

if wait "$build_pid"; then
    build_pid=""
    clear_progress
    # Record that setup finished, so an interrupted build can be told apart
    # from a complete one next time.
    : > "$BUILD_MARKER"
    say "All set! Starting microwriter..."
    say ""
    if [ -x "$RELEASE_BIN" ]; then
        exec "$RELEASE_BIN" "${SCRIPT_ARGS[@]}"
    fi
    # Fall back to cargo if the binary landed somewhere unexpected.
    exec "$CARGO" run --release --quiet -- "${SCRIPT_ARGS[@]}"
fi

build_pid=""
clear_progress

# The build did not finish. If this run has not already restarted once, throw
# the broken folder away and start over fresh; otherwise show the message.
if [ "$restarted" -eq 0 ]; then
    restart_fresh
fi

say ""
say "Sorry - setting microwriter up did not finish successfully."
say "The details were saved in this file:"
say ""
say "  $BUILD_LOG"
say ""
say "Sharing that file is all that is needed to get it sorted out."
say ""
pause_before_close
exit 1
