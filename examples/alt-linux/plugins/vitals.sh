#!/usr/bin/env bash
# Optimized for Minisforum (Ryzen APU) on ALT Linux
# Uses amdgpu PPT for power monitoring

# Получаем версию Mesa (только цифры)
mesa_v=$(glxinfo | grep "OpenGL version string" | awk -F'Mesa ' '{print $2}' | tr -d ')' | xargs)
mesa_v=${mesa_v:-"--"}

while true; do
    # 1. Сбор температур
    cpu_t=$(sensors k10temp-pci-00c3 2>/dev/null | awk '/Tctl/ {gsub(/\+|°C/,"",$2); print $2}')
    gpu_t=$(sensors amdgpu-pci-c500 2>/dev/null | awk '/edge/ {gsub(/\+|°C/,"",$2); print $2}')
    ssd_t=$(sensors nvme-pci-0100 2>/dev/null | awk '/Composite/ {gsub(/\+|°C/,"",$2); print $2}')

    # 2. Сбор потребления
    pwr_raw=$(sensors amdgpu-pci-c500 2>/dev/null | awk '/PPT:/ {print $2}')
    
    # 3. Валидация данных
    cpu_t=${cpu_t:-"--"}
    gpu_t=${gpu_t:-"--"}
    ssd_t=${ssd_t:-"--"}
    
    if [ -n "$pwr_raw" ]; then
        pwr_w=$(LC_NUMERIC=C printf "%.0f" "$pwr_raw")
    else
        pwr_w="0"
    fi

    # 4. Логика статуса
    alt="normal"
    if [[ "$cpu_t" != "--" ]]; then
        is_hot=$(echo "$cpu_t > 75" | bc -l)
        if [ "$is_hot" -eq 1 ]; then
            alt="hot"
        fi
    fi

    # 5. Вывод JSON
    # Формат: CPU 32C GPU 30C SSD 25C | 8W | 26.0.5
    echo "{\"text\": \"CPU ${cpu_t}C GPU ${gpu_t}C SSD ${ssd_t}C | ${pwr_w}W | ${mesa_v}\", \"alt\": \"${alt}\"}"

    sleep 3
done
