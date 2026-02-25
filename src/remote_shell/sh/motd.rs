pub const MOTD: &str = r#"
if [ ! -f "$HOME/.hushlogin" ]; then
    if [ -d /etc/update-motd.d ]; then
        for script in $(ls /etc/update-motd.d/ | sort); do
            path="/etc/update-motd.d/$script"
            if [ -x "$path" ]; then
                "$path" 2>/dev/null
            fi
        done
    fi

    if [ -f /etc/motd ] && [ -s /etc/motd ]; then
        cat /etc/motd
    fi

    last_login=$(last -1 -R "$USER" | head -1) 2> /dev/null
    if [ -n "$last_login" ]; then
        echo "Last login: $last_login"
    fi
fi

"#;
