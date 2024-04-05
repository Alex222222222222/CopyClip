if (tailwindcss --help); then
    tailwindcss --minify -c ./tailwind.config.js -o ./tailwind.css
else
    npx tailwindcss --minify -c ./tailwind.config.js -o ./tailwind.css
fi