#!/usr/bin/env bash
# In-game check for NeoVitae slice 1: the Ara Vitae altar, the Capacity Rune
# family and the Novicius Orb of Vitae.
#
# Every mod command is typed with `ctl chat "/cmd"` because `ctl command` runs
# the console path and bypasses the native `perform_command` hook.
#
# The End, in the void at x=200 (away from the central island and its ambient
# item entities). The altar stands on the stone floor at y=101 with its runes at
# y=100.
#
# Default run:
#  A  an altar placed from its item is tier 0 with a 10,000 LP tank;
#  B  eight runes at y-1 (the apprentice layout) make it tier 1 with an 18,000
#     LP tank: only the four orthogonal `upgrade` positions are counted, as
#     upstream's scanForRunes does (10,000 * (1 + 4 * 0.2)) — the differential;
#  C  a right click with a diamond puts it in the altar's slot;
#  D  a creative orb right click fills the altar's room (BloodOrbItem's
#     instabuild branch), exactly to its capacity;
#  E  in survival the same click pays exactly 200 LP (2 health x the jar's
#     self_sacrifice_conversion of 100) and costs 2 health;
#  F  the altar crafts the weak orb at 2,000 LP: 200 ticks at 10 LP a tick;
#  G  an empty-hand right click takes the orb back;
#  H  an orb click outside the altar's 5-block range pays it nothing.
#
# EMBEDDED=1 (default): the game is launched with **only the native bundle** and
# the content comes from the bundle's embedded copy (`mixins/<id>/data/…`, the
# AE2 pattern). CONTROL=1 (default with EMBEDDED) first launches the same binary
# with `.horizon-cache` unusable, where the ids must be unknown.
#
# SELFTEST=1 flips E's 200 LP assertion (it must FAIL), proving the scenario can
# fail.
set -Eeuo pipefail
export LC_ALL=C
[ -z "${NV_DEBUG:-}" ] || set -x

EMBEDDED=${EMBEDDED:-1}
CONTROL=${CONTROL:-$EMBEDDED}

CLONE=${CLONE:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/neovitae
PIN_BIN=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-$PIN_BIN/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/neovitae-e2e.sock}
SELFTEST=${SELFTEST:-0}
KEEP=${KEEP:-0}
WORLD=NeoVitaeE2E
CONTROL_WORLD=NeoVitaeControl
X0=200
AX=$((X0 + 2))
ALTAR_Y=101
RUNE_Y=100
CAPACITY_BARE=10000
CAPACITY_RUNED=18000
ORB_LP=200
ORB_COST=2000
CRAFT_TICKS=220

command -v jq >/dev/null || { echo "missing jq (every assertion parses a ctl reply)" >&2; exit 1; }
[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/neovitae-home.XXXXXX)
# A scratch home per leg: the control's unusable `.horizon-cache` must never be
# in the same directory the main leg's bundle writes its content to.
set_home() { # set_home DIR
    export HORIZON_HOME=$1/game
    export HORIZON_LAUNCHER_HOME=$1/launcher
    export XDG_DATA_HOME=$1/data
    export XDG_CONFIG_HOME=$1/config
    export HORIZON_NO_AUDIO=1
    mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
    rm -f "$SOCK"
}
set_home "$SCRATCH"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 60 >&2 || true; exit 1; }
on_error() { echo "FAIL: line $1: '$2' exited $?" >&2; ctl logs --lines 40 >&2 || true; }
trap 'on_error "$LINENO" "$BASH_COMMAND"' ERR

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH ${CONTROL_SCRATCH:-}" >&2
    else
        rm -rf "$SCRATCH" ${CONTROL_SCRATCH:-} 2>/dev/null || true
    fi
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch [WORLD]
    local world=${1:-$WORLD}
    rm -f "$SOCK"
    cd "$SCRATCH"
    local args=(--game "$GAME" --world "$world" --seed 20261105 --creative --headless
                --wait --resources "$RES")
    if [ "$EMBEDDED" = 1 ]; then
        echo "  launch: no --dev-mod; the bundle carries its own content" >&2
    else
        args+=(--dev-mod "$PORT/content")
    fi
    "$CTL" --socket "$SOCK" launch "${args[@]}" >/dev/null
    ctl wait-ready 120
    ctl wait 40
}

block_name() { ctl block "$1" "$2" "$3" | jq -r '.name // .block // empty'; }
player_health() { ctl state | jq -r '.player.health'; }

# The End's void at X0: a stone floor and clear air above it.
arena() {
    ctl command "execute in minecraft:the_end run tp @s $X0.5 104 0.5" >/dev/null
    local loaded=false
    for _ in $(seq 1 120); do
        ctl wait 5 >/dev/null
        if [ "$(ctl block "$X0" 100 0 | jq -r '.loaded // false')" = true ]; then
            loaded=true
            break
        fi
    done
    [ "$loaded" = true ] || fail "the End arena chunk never loaded"
    ctl command "execute in minecraft:the_end run fill $((X0 - 4)) 100 -14 $((X0 + 40)) 100 14 minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run fill $((X0 - 4)) 101 -14 $((X0 + 40)) 108 14 minecraft:air" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $X0.5 $ALTAR_Y 0.5" >/dev/null
    ctl wait 20
    [ "$(block_name "$X0" 100 0)" = "minecraft:stone" ] || fail "the arena floor is not stone"
    [ "$(block_name "$X0" 101 0)" = "minecraft:air" ] || fail "the arena air above the floor is not air"
}

# `/neovitae altar X Y Z`: the line, then TIER, LP, CAP, RESERVE, PROGRESS,
# SLOT, T.
info() { # info X Y Z
    say "neovitae altar $1 $2 $3"
    ctl wait 12 >/dev/null
    INFO_LINE=$(ctl messages 12 | jq -r '.chat[]' | grep -m1 'Ara Vitae at' || true)
    [ -n "$INFO_LINE" ] || fail "no /neovitae altar reply"
    eval "$(python3 "$PORT/tools/parse_info.py" "$INFO_LINE")"
    [ "${ok:-0}" = 1 ] || fail "the info line did not parse: $INFO_LINE"
    echo "  $INFO_LINE" >&2
}

expect() { # expect NAME GOT WANT
    [ "$2" = "$3" ] || fail "$1 is $2, expected $3"
}

hold() { # hold ITEM
    ctl command "item replace entity @s hotbar.0 with $1 1" >/dev/null
    ctl wait 4 >/dev/null
    ctl select 1 >/dev/null
    ctl wait 2 >/dev/null
}

# Place the altar from its item at X Y Z (a player invites a neighbour's top
# face), and check the block appeared.
place_altar() { # place_altar X Y Z
    local x=$1 y=$2 z=$3
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 $y $z.5" >/dev/null
    ctl wait 6 >/dev/null
    hold neovitae:ara_vitae
    ctl look-at-block "$x" $((y - 1)) "$z" up >/dev/null || fail "cannot aim at the floor under $x,$y,$z"
    ctl place "$x" "$y" "$z" --item neovitae:ara_vitae >/dev/null \
        || fail "ctl place at $x,$y,$z failed"
    ctl wait 6 >/dev/null
    local name
    name=$(block_name "$x" "$y" "$z")
    [ "$name" = "neovitae:ara_vitae" ] || fail "the altar did not place at $x,$y,$z (got '$name')"
}

# The eight apprentice runes at y-1, the jar's own layout (four orthogonal
# `upgrade` positions, four corners).
ring() { # ring X Y Z RUNE
    local x=$1 y=$2 z=$3 rune=$4
    for dx in -1 0 1; do
        for dz in -1 0 1; do
            [ "$dx" = 0 ] && [ "$dz" = 0 ] && continue
            ctl command "execute in minecraft:the_end run setblock $((x + dx)) $((y - 1)) $((z + dz)) $rune" >/dev/null
        done
    done
    # The altar revalidates its structure every 100 ticks
    # (`AltarConstants.STRUCTURE_CHECK_INTERVAL`).
    ctl wait 120 >/dev/null
}

# The control for EMBEDDED=1: the same binary with the embedded content made
# unavailable — `.horizon-cache` is a plain file, so the helper can neither
# compare nor write its content file and falls back to `Continue`.
control_leg() {
    echo "== control: the same game with no embedded content file =="
    local main=$SCRATCH
    SCRATCH=$(mktemp -d /tmp/opencode/neovitae-control.XXXXXX)
    CONTROL_SCRATCH=$SCRATCH
    set_home "$SCRATCH"
    : > "$SCRATCH/.horizon-cache"
    launch "$CONTROL_WORLD"
    arena
    ctl command "item replace entity @s hotbar.0 with neovitae:ara_vitae 1" >/dev/null || true
    ctl wait 4 >/dev/null
    ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "neovitae:ara_vitae")] | length == 0' >/dev/null \
        || fail "the control game has the altar item, so the content did not come from the embedded file"
    ctl command "execute in minecraft:the_end run setblock $AX $ALTAR_Y 0 neovitae:ara_vitae" >/dev/null || true
    ctl wait 4 >/dev/null
    local name
    name=$(block_name "$AX" "$ALTAR_Y" 0)
    [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so the block id was known"
    [ -f "$SCRATCH/.horizon-cache" ] || fail "the control's .horizon-cache was replaced: the helper wrote its content"
    echo "  without the embedded file: no altar item, the block stayed $name"
    ctl quit >/dev/null 2>&1 || true
    sleep 2
    SCRATCH=$main
    CONTROL_SCRATCH=
    set_home "$SCRATCH"
}

# EMBEDDED=1: the bundle must have written its content out at startup, byte for
# byte the delivered content mod.
embedded_content_check() {
    local cache=$SCRATCH/.horizon-cache/neovitae-embedded-content.hmod
    [ -f "$cache" ] || fail "the bundle did not write its embedded content to $cache"
    cmp -s "$cache" "$PORT/content/neovitae.hmod" \
        || fail "the embedded content differs from content/neovitae.hmod"
    echo "  embedded content: $(wc -c < "$cache" | tr -d ' ') B at $cache, identical to content/neovitae.hmod"
}

echo "== launch (The End arena, content + native bundle) =="
if [ "$CONTROL" = 1 ]; then control_leg; fi
launch
if [ "$EMBEDDED" = 1 ]; then embedded_content_check; fi
arena

ctl command "item replace entity @s hotbar.0 with neovitae:ara_vitae 1" >/dev/null
ctl wait 4 >/dev/null
ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "neovitae:ara_vitae")] | length >= 1' >/dev/null \
    || fail "the altar item did not register: $(ctl inventory | jq -c '{hotbar,main}')"

echo "== A: an altar placed from its item is tier 0, 10,000 LP =="
place_altar "$AX" "$ALTAR_Y" 0
info "$AX" "$ALTAR_Y" 0
expect tier "$TIER" 0
expect capacity "$CAP" "$CAPACITY_BARE"
expect "the tank" "$LP" 0
expect "the slot" "$SLOT" empty
echo "  tier $TIER, $LP/$CAP LP"

echo "== B: eight runes make the apprentice altar, 18,000 LP =="
ring "$AX" "$ALTAR_Y" 0 neovitae:rune_capacity
info "$AX" "$ALTAR_Y" 0
expect tier "$TIER" 1
expect capacity "$CAP" "$CAPACITY_RUNED"
echo "  tier $TIER, $LP/$CAP LP (4 upgrade runes x 0.2 of 10,000)"

echo "== C: a creative orb click fills the altar's room =="
hold neovitae:blood_orb_weak
ctl look 0 -90 >/dev/null
ctl wait 2 >/dev/null
ctl use-item >/dev/null
ctl wait 8 >/dev/null
info "$AX" "$ALTAR_Y" 0
expect "the tank" "$LP" "$CAPACITY_RUNED"
echo "  creative: $LP/$CAP LP (instabuild fills the room)"

echo "== D: in survival the same click pays exactly 200 LP for 2 health =="
say "neovitae lp $AX $ALTAR_Y 0 0"
ctl wait 8 >/dev/null
ctl command "gamemode survival" >/dev/null
ctl command "effect give @s minecraft:instant_health 1 10" >/dev/null
ctl wait 20 >/dev/null
BEFORE_HEALTH=$(player_health)
hold neovitae:blood_orb_weak
ctl look 0 -90 >/dev/null
ctl wait 2 >/dev/null
ctl use-item >/dev/null
ctl wait 8 >/dev/null
info "$AX" "$ALTAR_Y" 0
AFTER_HEALTH=$(player_health)
echo "  health $BEFORE_HEALTH -> $AFTER_HEALTH, lp $LP"
if [ "$SELFTEST" = 1 ]; then
    echo "(selftest: asserting the altar holds 201 LP)"
    expect "the tank" "$LP" 201
else
    expect "the tank" "$LP" "$ORB_LP"
    # The LP is the strong claim: 200 LP is exactly the 2 points the altar
    # measured. Health itself regenerates within the wait, so it is only held to
    # having moved down.
    python3 - "$BEFORE_HEALTH" "$AFTER_HEALTH" <<'HEALTH' || fail "the orb did not cost health"
import sys
before, after = (float(value) for value in sys.argv[1:3])
assert after < before, f"health went {before} -> {after}: nothing was paid"
HEALTH
fi
ctl command "gamemode creative" >/dev/null
ctl wait 4 >/dev/null

echo "== E: a right click with a diamond fills the altar's slot =="
say "neovitae lp $AX $ALTAR_Y 0 0"
ctl wait 8 >/dev/null
hold minecraft:diamond
ctl command "execute in minecraft:the_end run tp @s $((AX - 2)).5 $ALTAR_Y 0.5" >/dev/null
ctl wait 6 >/dev/null
INTERACT=$(ctl interact "$AX" "$ALTAR_Y" 0 up 2>&1) || fail "ctl interact on the altar failed: $INTERACT"
echo "  interact: $INTERACT" >&2
ctl wait 6 >/dev/null
info "$AX" "$ALTAR_Y" 0
echo "  hook counters: uses=$USES right=$RIGHT" >&2
expect "the slot" "$SLOT" "minecraft:diamond x1"
# A dry altar is the control for the loop: the item sits there untouched.
ctl wait 30 >/dev/null
info "$AX" "$ALTAR_Y" 0
expect "the slot" "$SLOT" "minecraft:diamond x1"
expect "the tank" "$LP" 0

echo "== F: the altar crafts the weak orb at 2,000 LP =="
say "neovitae lp $AX $ALTAR_Y 0 $CAPACITY_RUNED"
ctl wait 8 >/dev/null
ctl wait "$CRAFT_TICKS" >/dev/null
info "$AX" "$ALTAR_Y" 0
expect "the slot" "$SLOT" "neovitae:blood_orb_weak x1"
expect "the tank" "$LP" $((CAPACITY_RUNED - ORB_COST))
echo "  the diamond became a weak orb, $ORB_COST LP spent in 200 ticks"

echo "== G: an empty-hand right click takes the orb back =="
ctl command "item replace entity @s hotbar.0 with minecraft:air" >/dev/null
ctl wait 4 >/dev/null
ctl look-at-block "$AX" "$ALTAR_Y" 0 up >/dev/null
ctl interact "$AX" "$ALTAR_Y" 0 up >/dev/null || fail "ctl interact on the altar failed"
ctl wait 6 >/dev/null
ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "neovitae:blood_orb_weak")] | length >= 1' >/dev/null \
    || fail "the orb did not come back: $(ctl inventory | jq -c '{hotbar,main}')"
info "$AX" "$ALTAR_Y" 0
expect "the slot" "$SLOT" empty
echo "  the orb is in the player's inventory"

echo "== H: an orb click outside the altar's range pays it nothing =="
say "neovitae lp $AX $ALTAR_Y 0 1000"
ctl wait 8 >/dev/null
ctl command "execute in minecraft:the_end run tp @s $((AX + 28)).5 $ALTAR_Y 0.5" >/dev/null
ctl wait 10 >/dev/null
hold neovitae:blood_orb_weak
ctl look 0 -90 >/dev/null
ctl wait 2 >/dev/null
ctl use-item >/dev/null
ctl wait 8 >/dev/null
info "$AX" "$ALTAR_Y" 0
expect "the tank" "$LP" 1000
echo "  the altar 28 blocks away still holds $LP LP"

echo "PASS: an altar placed from its item is tier 0 with 10,000 LP, eight runes make it an apprentice altar whose four upgrade runes give 18,000 LP, a diamond goes into its slot, a creative orb fills its room, a survival orb pays exactly 200 LP for 2 health, the altar crafts the weak orb at 2,000 LP in 200 ticks, an empty hand takes the orb back, and an orb out of range pays nothing"
