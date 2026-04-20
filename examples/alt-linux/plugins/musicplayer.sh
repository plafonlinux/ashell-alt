#!/bin/bash
# Аргумент командной строки: 'check' для кнопок или 'info' для текста
MODE=$1 

while true; do
    # Проверяем, запущен ли плеер
    player=$(playerctl -l 2>/dev/null | head -n 1)
    
    if [ -z "$player" ]; then
        # Если плеера нет — выводим пустой текст с валидным alt
        echo '{"text": "", "alt": "stopped"}'
    else
        if [ "$MODE" == "check" ]; then
            # Для кнопок Prev/Next отправляем валидный JSON
            echo '{"text": " ", "alt": "active"}'
        else
            # Основная логика вывода текста трека
            status=$(playerctl --player="$player" status 2>/dev/null)
            artist=$(playerctl --player="$player" metadata xesam:artist 2>/dev/null || echo "Unknown")
            title=$(playerctl --player="$player" metadata xesam:title 2>/dev/null || echo "Track")
            
            # Статусная иконка (Play/Pause)
            icon="󰐊"; [ "$status" = "Paused" ] && icon="󰏤"
            
            # Формируем строку и ограничиваем длину
            display="$artist — $title"
            [ ${#display} -gt 45 ] && display="${display:0:42}..."
            
            echo "{\"text\": \"$icon $display\", \"alt\": \"$status\"}"
        fi
    fi
    sleep 2
done
