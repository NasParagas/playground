const music = Application("Music");
if (music.playerState() === "playing") {
    const t = music.currentTrack;
    JSON.stringify({
        name: t.name(),
        artist: t.artist(),
        album: t.album(),
        position: music.playerPosition(),
        duration: t.duration(),
    });
} else {
    JSON.stringify({ playing: false });
}
