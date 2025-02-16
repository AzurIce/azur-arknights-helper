update-resources:
    if [[ {{os()}} == "windows" ]]; then \
        powershell -Command "echo \"last_updated = \`\"\$((Get-Date).ToString(\"yyyy-MM-ddTHH:mm:ss.fffffffzzz\"))\`\"\"" > resources/manifest.toml; \
    else \
        echo "last_updated = $(date --rfc-3339=seconds)" > resources/manifest.toml; \
    fi
    cargo build-resources