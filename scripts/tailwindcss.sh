if command -v tailwindcss >/dev/null 2>&1; then
    tailwindcss --minify -c ./tailwind.config.js -o ./tailwind.css
else
    npx tailwindcss --minify -c ./tailwind.config.js -o ./tailwind.css
fi
