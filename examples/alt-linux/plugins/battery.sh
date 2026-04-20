#!/usr/bin/env bash
# Battery listen_cmd for ashell custom module
# Outputs Waybar-format JSON: { "text": "...", "alt": "..." }
# Format: {power}W / {capacity}% / {time}
# Example: 5.02W / 95% / 10h 1min

BATTERY_PATH="/sys/class/power_supply"

get_battery() {
    for bat in "$BATTERY_PATH"/BAT*; do
        [ -d "$bat" ] && echo "$bat" && return
    done
    echo ""
}

format_time() {
    local minutes=$1
    if [ "$minutes" -le 0 ] 2>/dev/null; then
        echo "∞"
        return
    fi
    local hours=$(( minutes / 60 ))
    local mins=$(( minutes % 60 ))
    if [ "$hours" -gt 0 ] && [ "$mins" -gt 0 ]; then
        echo "${hours}h ${mins}min"
    elif [ "$hours" -gt 0 ]; then
        echo "${hours}h"
    else
        echo "${mins}min"
    fi
}

get_battery_info() {
    local bat
    bat=$(get_battery)

    if [ -z "$bat" ]; then
        printf '{"text": "No battery", "alt": "unknown"}\n'
        return
    fi

    local capacity status power_uw power_w time_str alt

    capacity=$(cat "$bat/capacity" 2>/dev/null || echo "0")
    status=$(cat "$bat/status" 2>/dev/null || echo "Unknown")
    power_uw=0
    power_w="0.00"

    # --- Power (Watts) ---
    if [ -f "$bat/power_now" ]; then
        power_uw=$(cat "$bat/power_now" 2>/dev/null || echo "0")
        power_w=$(awk "BEGIN { printf \"%.2f\", $power_uw / 1000000 }")
    elif [ -f "$bat/current_now" ] && [ -f "$bat/voltage_now" ]; then
        local current_ua voltage_uv
        current_ua=$(cat "$bat/current_now" 2>/dev/null || echo "0")
        voltage_uv=$(cat "$bat/voltage_now" 2>/dev/null || echo "0")
        power_uw=$(awk "BEGIN { printf \"%d\", ($current_ua * $voltage_uv) / 1000000 }")
        power_w=$(awk "BEGIN { printf \"%.2f\", $power_uw / 1000000 }")
    fi

    # --- Time remaining ---
    case "$status" in
        Discharging)
            alt="discharging"
            if [ -f "$bat/energy_now" ] && [ -f "$bat/power_now" ] && [ "$power_uw" -gt 0 ]; then
                local energy_now
                energy_now=$(cat "$bat/energy_now")
                local minutes
                minutes=$(awk "BEGIN { printf \"%d\", ($energy_now / $power_uw) * 60 }")
                time_str=$(format_time "$minutes")
            elif [ -f "$bat/charge_now" ] && [ -f "$bat/current_now" ]; then
                local charge_now current_now
                charge_now=$(cat "$bat/charge_now")
                current_now=$(cat "$bat/current_now")
                if [ "$current_now" -gt 0 ]; then
                    local minutes
                    minutes=$(awk "BEGIN { printf \"%d\", ($charge_now / $current_now) * 60 }")
                    time_str=$(format_time "$minutes")
                else
                    time_str="∞"
                fi
            else
                time_str="∞"
            fi
            ;;
        Charging)
            alt="charging"
            if [ -f "$bat/energy_full" ] && [ -f "$bat/energy_now" ] && [ "$power_uw" -gt 0 ]; then
                local energy_full energy_now energy_left
                energy_full=$(cat "$bat/energy_full")
                energy_now=$(cat "$bat/energy_now")
                energy_left=$(( energy_full - energy_now ))
                local minutes
                minutes=$(awk "BEGIN { printf \"%d\", ($energy_left / $power_uw) * 60 }")
                time_str=$(format_time "$minutes")
            elif [ -f "$bat/charge_full" ] && [ -f "$bat/charge_now" ] && [ -f "$bat/current_now" ]; then
                local charge_full charge_now current_now charge_left
                charge_full=$(cat "$bat/charge_full")
                charge_now=$(cat "$bat/charge_now")
                current_now=$(cat "$bat/current_now")
                if [ "$current_now" -gt 0 ]; then
                    charge_left=$(( charge_full - charge_now ))
                    local minutes
                    minutes=$(awk "BEGIN { printf \"%d\", ($charge_left / $current_now) * 60 }")
                    time_str=$(format_time "$minutes")
                else
                    time_str="∞"
                fi
            else
                time_str="∞"
            fi
            ;;
        Full)
            alt="full"
            power_w="0.00"
            time_str="Full"
            ;;
        *)
            alt="unknown"
            time_str="∞"
            ;;
    esac

    printf '{"text": "%sW / %s%% / %s", "alt": "%s"}\n' \
        "$power_w" "$capacity" "$time_str" "$alt"
}

# Loop forever — ashell's listen_cmd expects continuous stdout
while true; do
    get_battery_info
    sleep 2
done
