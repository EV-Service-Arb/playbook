# capture

Code does this step; nothing here is written by hand.

- a source enters by a line in `knowledge/references/README.md` and its platform's puller:
  `cargo r -p playbook -- pull <platform>`. Never a capture written any other way, and never a link a
  puller printed followed without registering it
- `pull yt transcribe` spends per video, and with no arguments takes the whole unticked backlog. Name
  the videos
- then the picture: `cargo r -p playbook -- watch <capture>/src.md` writes `<capture>/shown.md`, looking only
  where the speech says something is shown. It spends per recording, and with no arguments takes every
  unwatched one. Name the captures
- a recording itself is reached only through `cargo r -p playbook -- recording <capture>/src.md`
- a channel or a speaker not yet known: [people.md](people.md)
