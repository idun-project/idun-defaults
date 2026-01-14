#!/bin/bash
# Idun's default .bashrc, Copyright© 2025 Brian Holdsworth
# This is free software, released under the MIT License.
#
# This application provides a custom set of batch routines.
# With the help of the 'idunsh' program, these routines can
# launch software on the Commodore CPU from any Linux terminal.
# It is by default used with 'shell.app' and the terminal is
# run and displayed by the Commodore.
#
# With slightly different configuration managed automatically
# below, these routines work from any kind of Linux Bash shell
# console, screen, window, or remote connection.
#
# This script assumes the cartridge is physically connected to 
# the Commodore and that the `shell.app` has been started.

# If not running interactively, don't do anything
[[ $- != *i* ]] && return

# Regular aliases
alias ls='ls --color=auto'
alias grep='grep --color=auto'

# Use append for bash history
shopt -s histappend

# Global variable used with run command
FF_LAST_MATCH=

# This routine provides a safe wrapper around idunsh
idunshell() {
  history -a
  exec idunsh "$@" || {
    echo "idunsh failed to load" >&2
    return 127
  }
}

# This routine wraps exec commands, and redirects output
# when the command is being run from a standard Linux prompt
# instead of form within the shell.app.
idunexec() {
  if [[ $(< /proc/$PPID/comm) == "idunio" ]]; then
    idunshell -s exec "$@"
  else
    idunsh -s -o exec "$@"
  fi
}

# This routine wraps non-exec commands, and redirects output.
idunmsg() {
  if [[ $(< /proc/$PPID/comm) == "idunio" ]]; then
    idunshell "$@"
  else
    idunsh -o "$@"
  fi
}

# Show command help using Lua program
help() {
  idunexec tty l:help.lua $
}

W='\[\033[37m\]'    # bright white foreground
G='\[\033[32m\]'    # green foreground
N='\[\033[0m\]'     # reset colors to default
PS1="${W}\u ${N} ${G}\W ${N}\$ "

# ---------------------------------------------------------------------------
# INTERNAL UTILITIES
# ---------------------------------------------------------------------------

_mluasend() {
    echo "$1" | socat - UNIX-CONNECT:/tmp/idunmm-lua
}
# Alias for system reboot
alias reboot='_mluasend "sys.reboot(0)"'

_fname() {
    local arg=$1

    [[ -z $arg ]] && return 1

    # If it begins with a-z or A-Z followed by a colon, return it unchanged
    if [[ $arg =~ ^[A-Za-z]: ]]; then
        return 0
    fi

    # Expand ~ and normalize
    arg=${arg/#\~/$HOME}

    [[ -f "$arg" ]]
}

_toolhdr() {
    local file=$1
    local b

    [[ -r $file ]] || return 1

    # Read first 8 bytes as hex octets
    read -r -a b < <(
        od -An -tx1 -N8 -- "$file"
    )

    # Must have exactly 8 bytes
    (( ${#b[@]} == 8 )) || return 1

    # Match: 4c xx xx cb 06 10 40 00
    [[ ${b[0]} == 4c &&
       ${b[3]} == cb &&
       ${b[4]} == 06 &&
       ${b[5]} == 10 &&
       ${b[6]} == 40 &&
       ${b[7]} == 00 ]]
}

# ---------------------------------------------------------------------------
# Handler to allow easy running of any Idun programs
# ---------------------------------------------------------------------------
command_not_found_handle() {
    local cmd="$1"
    shift

    # Attempt glob expansion
    local expanded=()
    for arg in "$@"; do
        # If the glob matches, expand; else keep original
        if compgen -G "$arg" > /dev/null; then
            expanded+=( $arg )    # unquoted = expand
        else
            expanded+=( "$arg" )  # keep literal
        fi
    done

    # If command found in Z:, then execute
    # else if includes a device prefix, try to execute
    # else if a file in the current directory with a
    # valid Idun Tool header, try to execute
    if [[ -f "${IDUN_SYS_DIR}/sys/${cmd}" ]]; then
        idunexec "$cmd" "${expanded[@]}"
    elif [[ $cmd =~ ^[A-Za-z]: ]]; then
        idunexec "$cmd" "${expanded[@]}"
    elif _toolhdr "./$cmd"; then
        idunexec "$cmd" "${expanded[@]}"
    else
        printf '%s: command not found\n' "$cmd" >&2
    fi

    return 127
}

# ---------------------------------------------------------------------------
# EMBEDDED LUA
# ---------------------------------------------------------------------------

# Run a Lua script through Idun's integral Lua interpreter
mlua() {
    # Usage: mlua luafile
    local file="$1"
    [[ -z "$file" ]] && { idunexec tty "l:"; return; }

    # Append .lua if not present
    [[ "$file" != *.lua ]] && file="${file}.lua"

    # Run the command
    idunexec tty "l:${file}"
}

# ---------------------------------------------------------------------------
# LISTING AND MOUNTING DRIVES
# ---------------------------------------------------------------------------

# List virtual drives. Also change their mounts.
drives() {
    # Usage: drives [<drive>:] [path-or-image]
    # e.g. "drives d: mydisk.d64"
    #      "drives e: ~/projects/goodies"
    # No arguments → show drives
    if [[ $# -eq 0 ]]; then
        idunmsg drives
        return
    fi

    # Must have exactly 2 arguments
    if [[ $# -ne 2 ]]; then
        echo "Usage: drives [a-z]: path-or-image"
        return 1
    fi

    local drive="$1"
    local target="$2"

    # Validate drive format: single letter + colon
    if [[ ! "$drive" =~ ^[A-Za-z]:$ ]]; then
        echo "Error: First argument must be a single letter followed by a colon (e.g. d:)"
        return 1
    fi

    # If trying to mount to a: or b:, then only allow for
    # C64 Ultimate and only if configured with the IP.
    if [[ "$drive" =~ ^[aAbB]:$ ]] && [[ -n $C64_ULTIMATE_IP ]]; then
        idunsh -u mount "$drive" "$target"
        return
    fi

    # Handle .d64 / .d71 / .t64 images
    if [[ "$target" =~ \.(d(64|71)|t64)$ ]]; then
        idunmsg -s mount "$drive" "$target"
        return
    fi

    # Otherwise, check that it’s a valid path
    if [[ -d "$target" ]]; then
        idunmsg -s assign "$drive" "$target"
    else
        echo "Error: Path not found: $target"
        return 1
    fi
}

# ---------------------------------------------------------------------------
# DIRECTORY
# ---------------------------------------------------------------------------

# Get a directory of a virtual drive
dir() {
    # Usage: dir <drive>:
    local arg="$1"

    # If no argument given, just call normal dir
    if [[ -z $arg ]]; then
        command dir         # use `command` to avoid recursion
        return $?
    fi

    # If argument starts with letter + colon (e.g., C:, Z:)
    if [[ $arg =~ ^[A-Za-z]: ]]; then
        idunmsg -s dir "$arg"
        return $?
    else
        command dir "$arg"  # otherwise call normal dir
        return $?
    fi
}

# Get a directory of virtual drive using Commodore format
catalog() {
  # Usage: catalog <drive>:
  local arg="$1"

  # If no argument given, then catalog c:
  if [[ -z $arg ]]; then
      arg="c:"
  fi

  if [[ $(< /proc/$PPID/comm) == "idunio" ]]; then
    idunshell --xarg=p catalog "$arg"
  else
    idunsh -o catalog "$arg"
  fi
}

# ---------------------------------------------------------------------------
# STARTING PROGRAMS
# ---------------------------------------------------------------------------

# Launch an Idun app
go() {
  # Usage: go appname -or- go <drive>:appname
  local app="$1"
  
  # 'go' command is for .app's. Add extension if not present.
  if [[ $app != *.app ]]; then
    app="${app}.app"
  fi

  if result=$(_fname "$app"); then
    idunmsg -s go "$app"
  elif [[ -f "${IDUN_SYS_DIR}/sys/${app}" ]]; then
    idunmsg go "z:${app}"
  else
    printf 'Error: File not found\n' >&2
  fi
}

# Load and run a Commodore PRG or launch other content on
# a C64 Ultimate.
run() {
  local opt_u=0
  local filename

  # Parse optional -u switch
  if [[ $1 == "-u" ]]; then
    opt_u=1
    shift
  fi

  # Check filename is reasonable
  filename=$1
  if ! result=$(_fname "$filename"); then
    printf 'Error: File not found\n' >&2
    return 2
  fi

  # Extract lowercase extension (if any)
  local ext=${filename##*.}
  ext=${ext,,}

  case "$ext" in
    sid|mod|crt)
      idunmsg -u load "$filename"
      ;;
    *)
      if (( opt_u )); then
        idunmsg -u load "$filename"
      else
        idunmsg -s load "$filename"
      fi
      ;;
  esac
}

# Load and run a z80 program
zload() {
  # Usage: zload z80prg -or- zload <drive>:z80prg
  local prg="$1"
  
  if result=$(_fname "$prg"); then
    idunexec zload "$prg"
  else
    printf 'Error: File not found\n' >&2
  fi
}

# Show picture files
show() {
    # Show image files (.koa, .scr, .vdc) using the correct Idun viewer.
    # Usage: show <file1> [file2 ...]
    # If all args share the same extension among .koa/.scr/.vdc, pick the viewer:
    #   .koa -> idunexec showkoa
    #   .scr -> idunexec showzx
    #   .vdc -> idunexec showvdc
    # Otherwise (mixed/no/unrecognized ext) -> idunexec showvdc
    if [[ $# -eq 0 ]]; then
        echo "Usage: show <file1> [file2 ...]" >&2
        return 1
    fi

    local allowed_koa="koa" allowed_scr="scr" allowed_vdc="vdc"
    local ext
    local uniq_exts=()   # unique recognized extensions among args (.koa/.scr/.vdc)
    local saw_other=0    # set if an argument has no extension or an unrecognized extension

    # helper to add unique extension to uniq_exts
    _add_ext() {
        local e="$1"
        for i in "${uniq_exts[@]}"; do
            [[ "$i" == "$e" ]] && return
        done
        uniq_exts+=("$e")
    }

    for f in "$@"; do
        if [[ "$f" =~ \.([^.]+)$ ]]; then
            ext="${BASH_REMATCH[1],,}"   # lowercase extension
            case "$ext" in
                koa|scr|vdc) _add_ext "$ext" ;;
                *)
                    # unrecognized extension -> treat as "other"
                    saw_other=1
                    ;;
            esac
        else
            # no extension -> treat as "other"
            saw_other=1
        fi
    done

    # Decide which subcommand to use
    local subcmd="showvdc"   # default
    if [[ $saw_other -eq 0 && ${#uniq_exts[@]} -eq 1 ]]; then
        case "${uniq_exts[0]}" in
            koa) subcmd="showkoa" ;;
            scr) subcmd="showzx" ;;
            vdc) subcmd="showvdc" ;;
        esac
    else
        # mixed/no/unrecognized extension -> default to showvdc (per spec)
        subcmd="showvdc"
    fi

    # Build and run the final command. Use printf '%q' to properly quote arguments.
    # Example final eval string: idunexec showkoa 'file1.koa' 'file2.koa'
    local quoted_args
    quoted_args=$(printf ' %q' "$@")   # leading space included
    local cmdline="idunexec ${subcmd}${quoted_args}"

    # Execute and preserve exit status
    eval "$cmdline"
    return $?
}

# Restart cartridge in BASIC mode
basic() {
    # If IDUN_SYS is unset or empty, do nothing
    [[ -z "$IDUN_SYS" ]] && return
    
    # Extract sys from IDUN_SYS="sys;banks"
    local sys="${IDUN_SYS%%;*}"

    # Send the command
    _mluasend "sys.reboot(${sys})"
}

# Add <TAB> filename completion. The filename comes from the last
# result of the ff (find file) command.
_ff_last_complete() {
    local cur
    cur=${COMP_WORDS[COMP_CWORD]}

    # Only offer completion if we have a last ff match
    [[ -n $FF_LAST_MATCH ]] || return

    # Only complete the first non-option argument
    local first_nonopt=1
    for ((i=1; i<COMP_CWORD; i++)); do
        [[ ${COMP_WORDS[i]} != -* ]] && first_nonopt=0
    done
    (( first_nonopt == 0 )) && return

    # Determine the string to insert, quoting if necessary
    match=$FF_LAST_MATCH
    if [[ $match == *[[:space:]]* ]]; then
        # Wrap in single quotes, escaping embedded single quotes
        match="'${match//\'/\'\\\'\'}'"
    fi
    COMPREPLY=("$match")
}
complete -F _ff_last_complete run
complete -F _ff_last_complete show
complete -F _ff_last_complete zload

# lightweight non-interactive fzf helpers for bash
# Requires: fzf, fd (https://github.com/sharkdp/fd)

# ---------------------------------------------------------------------------
# CONFIGURATION
# ---------------------------------------------------------------------------

# Base directory for searches
: "${FZF_LITE_HOME:=$HOME}"

# Cache directory
: "${FZF_LITE_CACHE:=$HOME/.cache/fzf-lite}"

mkdir -p "$FZF_LITE_CACHE"

# Cache files
FZF_LITE_FILE_CACHE="$FZF_LITE_CACHE/files.txt"
FZF_LITE_DIR_CACHE="$FZF_LITE_CACHE/dirs.txt"

# Refresh interval (seconds) — 6 hours default
: "${FZF_LITE_CACHE_TTL:=21600}"

# ---------------------------------------------------------------------------
# INTERNAL UTILITIES
# ---------------------------------------------------------------------------

# Non-interactive fuzzy filter
_fzf_filter() {
    local pattern="$1"
    fzf --filter "$pattern" --ignore-case
}

# Check if cache file is fresh (based on TTL)
_cache_is_fresh() {
  [[ -f "$1" ]] && (( $(date +%s) - $(stat -c %Y "$1") < $FZF_LITE_CACHE_TTL ))
}

# Refresh directory cache using fd
_refresh_dir_cache() {
  echo "Refreshing directory cache for $FZF_LITE_HOME..." >&2
  fd --type d --hidden --exclude .git . "$FZF_LITE_HOME" > "$FZF_LITE_DIR_CACHE"
}

# Refresh file cache using fd
_refresh_file_cache() {
  echo "Refreshing file cache for $FZF_LITE_HOME..." >&2
  fd --type f --hidden --exclude .git . "$FZF_LITE_HOME" > "$FZF_LITE_FILE_CACHE"
}

# Ensure both caches exist and are fresh
_ensure_cache() {
  _cache_is_fresh "$FZF_LITE_DIR_CACHE" || _refresh_dir_cache
  _cache_is_fresh "$FZF_LITE_FILE_CACHE" || _refresh_file_cache
}

# ---------------------------------------------------------------------------
# COMMANDS
# ---------------------------------------------------------------------------

# fcd — fuzzy cd into a directory
fcd() {
  local pattern="${1:-}"
  _ensure_cache
  local match
  match=$( <"$FZF_LITE_DIR_CACHE" _fzf_filter "$pattern" | head -n1)
  if [[ -n "$match" ]]; then
    cd "$match" || return
    echo "→ $(pwd)"
  else
    echo "No matching directory for '$pattern'" >&2
    return 1
  fi
}

# ff — fuzzy find file under FZF_LITE_HOME
ff() {
  # Usage: ff pattern
  local pattern="$1"
  local abs match rel

  _ensure_cache

  # Get absolute path first
  abs=$(_fzf_filter "$pattern" < "$FZF_LITE_FILE_CACHE" | head -n1) || return

  # Strip trailing whitespace
  abs=${abs//$'\r'/}
  abs=${abs//$'\n'/}
  abs=${abs//$'\t'/}

  # Convert to relative path if inside $PWD
  if [[ $abs == "$PWD"* ]]; then
    rel=${abs#$PWD/}
  else
    # fallback: leave as absolute if outside PWD
    rel=$abs
  fi

  # Store the relative path for later
  FF_LAST_MATCH=$rel

  # Print relative path
  printf '%s\n' "$rel"
}

# ---------------------------------------------------------------------------
# MAINTENANCE COMMANDS
# ---------------------------------------------------------------------------

# Refresh caches manually
fzf_cache_refresh() {
  echo "Rebuilding both caches..." >&2
  _refresh_dir_cache
  _refresh_file_cache
  echo "Caches updated." >&2
}

# Show cache stats
fzf_cache_info() {
  echo "fzf-lite cache info:"
  echo "  Home:   $FZF_LITE_HOME"
  echo "  Dir cache:  $(wc -l < "$FZF_LITE_DIR_CACHE" 2>/dev/null || echo 0) entries"
  echo "  File cache: $(wc -l < "$FZF_LITE_FILE_CACHE" 2>/dev/null || echo 0) entries"
  echo "  Cache age (dir): $(( $(date +%s) - $(stat -c %Y "$FZF_LITE_DIR_CACHE" 2>/dev/null || echo 0) ))s"
  echo "  Cache age (file): $(( $(date +%s) - $(stat -c %Y "$FZF_LITE_FILE_CACHE" 2>/dev/null || echo 0) ))s"
}
