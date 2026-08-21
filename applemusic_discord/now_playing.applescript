tell application "Music"
    if player state is playing then
        set t to current track
		return (name of t) & " — " & (artist of t) & " (" & (album of t) & ")"
	else
		return "not playing"
	end if
end tell
