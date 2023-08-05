# this file is both a valid
# - overlay which can be loaded with `overlay use starship.nu`
# - module which can be used with `use starship.nu`
# - script which can be used with `source starship.nu`
export-env { load-env {
    STARSHIP_SHELL: "nu"
    STARSHIP_SESSION_KEY: (random chars -l 16)
    PROMPT_MULTILINE_INDICATOR: (
        ^::STARSHIP:: prompt --continuation --disable-add-newline
    )

    # Does not play well with default character module.
    # TODO: Also Use starship vi mode indicators?
    PROMPT_INDICATOR: ""
    STARSHIP_FIRST_RENDER: 1

    PROMPT_COMMAND: {||
        # jobs are not supported
        (
            ^::STARSHIP:: prompt
                --cmd-duration $env.CMD_DURATION_MS
                $"--status=($env.LAST_EXIT_CODE)"
                --terminal-width (term size).columns
                --disable-add-newline
        )
    }

    config: ($env.config? | default {} | upsert hooks {
        # todo: make this not override existing hook
        # todo: doesn't work nice on windows for ctrl+c
        pre_prompt: {
            if ($env.STARSHIP_FIRST_RENDER == 1) {
                $env.STARSHIP_FIRST_RENDER = 0;
            } else if (^starship print-config add_newline | str contains "add_newline = true") {
                print "\n"
            }
        }
    } | merge {
        render_right_prompt_on_last_line: true,
    })

    PROMPT_COMMAND_RIGHT: {||
        (
            ^::STARSHIP:: prompt
                --right
                --cmd-duration $env.CMD_DURATION_MS
                $"--status=($env.LAST_EXIT_CODE)"
                --terminal-width (term size).columns
                --disable-add-newline
        )
    }
}}
