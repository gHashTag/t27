#!/bin/bash
# Скрипт для нативной оркестрации web-среды OpenCode

echo "🚀 Запуск веб-верии OpenCode в режиме Native Development (HMR & Vite)"
echo "=> Все изменения исходного кода UI (packages/app) будут загружаться налету."

# Убеждаемся что зависимости установлены
if [ ! -d "node_modules" ]; then
    echo "⚙️ Установка зависимостей (Bun)..."
    bun install
fi

echo "✅ Запуск 'bun --bun run dev' в packages/app..."
# Запускаем Vite через собственный рантайм `bun`, так как системный Node.js слишком старый для Vite 7
cd packages/app && bun --bun run dev
