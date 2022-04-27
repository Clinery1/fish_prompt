function preexec --on-event fish_preexec
    $HOME/projects/fish_prompt/target/release/fish_prompt $COLUMNS $LINES --preexec $argv[1]
end
function prompt --on-event fish_prompt
    #Save the return status of the previous command
    set last_pipestatus $pipestatus
    set PIPE_STATUS (
        for last_status in $last_pipestatus
            echo -n (fish_status_to_signal $last_status)
            echo -n " "
        end
    )
    $HOME/projects/fish_prompt/target/release/fish_prompt $COLUMNS $LINES --prompt "$PIPE_STATUS"
end
function fish_prompt --description 'Informative prompt'
    $HOME/projects/fish_prompt/target/release/fish_prompt $COLUMNS $LINES   # do the single-line prompt
end
