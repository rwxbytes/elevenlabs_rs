//! A karaoke-style radio drama using Text-to-Dialogue **with timestamps**.
//!
//! This example:
//!   1. Writes a short, emotionally-charged scene for three voices, using
//!      Eleven v3 audio tags (`[whispering]`, `[scoffs]`, `[laughs]`, ...).
//!   2. Generates the dialogue with character-level timing via
//!      `TextToDialogueWithTimestamps`.
//!   3. Plays the audio while streaming a colour-coded, word-by-word transcript
//!      to the terminal, following the model's reported `voice_segments` timing.
//!      Note: the API's timing can compress on longer dialogues, so the reveal is
//!      clamped to advance monotonically (see `build_timeline`) — it tracks the
//!      audio closely up front and degrades gracefully rather than bursting.
//!   4. Exports an `.lrc` subtitle file and the `.mp3`.
//!
//! Run with your API key set:
//!   ELEVENLABS_API_KEY=... cargo run -p dialogue_karaoke
//!
//! Docs:
//!   - https://elevenlabs.io/docs/overview/capabilities/text-to-dialogue
//!   - https://elevenlabs.io/docs/overview/capabilities/text-to-speech/best-practices

use elevenlabs_rs::endpoints::genai::text_to_dialogue::*;
use elevenlabs_rs::utils::{play, save};
use elevenlabs_rs::{DefaultVoice, ElevenLabsClient, Result, VoiceSettings};

use std::io::{stdout, Write};
use std::time::{Duration, Instant};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

/// A speaker in our scene: a display name, an ANSI colour, and the voice that
/// performs their lines.
struct Speaker {
    name: &'static str,
    color: &'static str,
    voice: DefaultVoice,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = ElevenLabsClient::from_env()?;

    // Cast the scene. Each speaker gets a distinct voice and terminal colour.
    let cast = [
        Speaker {
            name: "NOVA",
            color: "\x1b[96m",
            voice: DefaultVoice::Sarah,
        }, // hacker, cool
        Speaker {
            name: "KAINE",
            color: "\x1b[93m",
            voice: DefaultVoice::Brian,
        }, // muscle, gruff
        Speaker {
            name: "ECHO",
            color: "\x1b[95m",
            voice: DefaultVoice::Will,
        }, // rookie, jittery
    ];

    // The script: (speaker index, line). Audio tags steer Eleven v3's delivery.
    // Kept well under the 2,000-character Text-to-Dialogue limit.
    let script: &[(usize, &str)] = &[
        (
            2,
            "[whispering] Nova... are you sure the vault's on this floor?",
        ),
        (
            0,
            "[confidently] Relax. I rerouted the cameras myself. Ninety seconds, in and out.",
        ),
        (
            1,
            "[gruffly] Ninety seconds? [scoffs] I've cracked tougher safes in my sleep.",
        ),
        (
            2,
            "[nervously] That— that's not as reassuring as you think, Kaine.",
        ),
        (
            0,
            "[whispering] Quiet. Someone's coming. [pause] Down. Now!",
        ),
        (
            1,
            "[whispering] ...False alarm. Just a guard doing his rounds.",
        ),
        (
            2,
            "[sighs] I can't believe I let you two talk me into this.",
        ),
        (
            0,
            "[laughs] Too late to back out now, rookie. The vault's open.",
        ),
        (1, "[elated] Well, would you look at that. Payday."),
        (
            2,
            "[whispering, awed] We're... we're actually going to make it.",
        ),
    ];

    // Build the dialogue inputs from the script.
    let inputs: Vec<DialogueInput> = script
        .iter()
        .map(|(who, line)| DialogueInput::new(*line, String::from(cast[*who].voice.clone())))
        .collect();

    // Eleven v3 is the model behind Text-to-Dialogue. A lower stability ("Creative")
    // makes the audio tags and emotion land harder — ideal for a dramatic scene.
    let body = TextToDialogueBody::new(inputs)
        .with_model_id("eleven_v3")
        .with_settings(VoiceSettings::default().with_stability(0.4));

    println!(
        "{BOLD}🎬  THE LAST NINETY SECONDS{RESET}  {DIM}— generating with timestamps...{RESET}\n"
    );

    let resp = client.hit(TextToDialogueWithTimestamps::new(body)).await?;

    // Persist artifacts.
    let audio = resp.audio()?;
    save("heist.mp3", audio.clone())?;
    let lrc = build_lrc(&resp, &cast);
    std::fs::write("heist.lrc", &lrc)?;

    // `voice_segments` carry the real per-line audio times; build a word-level
    // timeline from them. (The character-level `alignment` is kept only for the
    // line text and the `.lrc` — its absolute times are unreliable here.)
    let cues = build_timeline(&resp, &cast);

    // Start the audio on a blocking thread; the main task drives the transcript
    // off a single wall-clock, revealing each word at its scheduled time.
    let clock = Instant::now();
    let playback = tokio::task::spawn_blocking(move || play(audio));

    let mut current_line: Option<usize> = None;
    let mut out = stdout();

    for cue in &cues {
        // Wait until this word is actually spoken, then reveal it.
        wait_until(clock, cue.start).await;

        // New dialogue turn -> new line with a speaker label.
        if Some(cue.line) != current_line {
            current_line = Some(cue.line);
            let stamp = timestamp(cue.start);
            write!(
                out,
                "\n{DIM}{stamp}{RESET} {}{BOLD}{:<5}{RESET} {DIM}│{RESET} ",
                cue.color, cue.name
            )
            .ok();
        }

        write!(out, "{}{}{RESET} ", cue.color, cue.text).ok();
        out.flush().ok();
    }
    println!("\n");

    // Make sure playback finished cleanly.
    playback.await.map_err(|e| e.to_string())??;

    // Closing summary.
    let total = resp
        .voice_segments
        .iter()
        .map(|s| s.end_time_seconds)
        .fold(0.0_f64, f64::max);
    println!(
        "{DIM}────────────────────────────────────────{RESET}\n\
         🎧  {BOLD}{} words{RESET} across {BOLD}{} lines{RESET} in {BOLD}{:.1}s{RESET}\n\
         💾  saved {BOLD}heist.mp3{RESET} and subtitles {BOLD}heist.lrc{RESET}",
        cues.len(),
        resp.voice_segments.len(),
        total,
    );

    Ok(())
}

/// One word to reveal in the transcript, with the audio time it should appear.
struct Cue {
    start: f64,
    text: String,
    name: &'static str,
    color: &'static str,
    /// Which dialogue turn this word belongs to (drives line breaks).
    line: usize,
}

/// Turn the response into a word-level reveal timeline.
///
/// Timing is derived from `voice_segments`, but defensively: the API's absolute
/// `start_time_seconds` can *compress* on longer dialogues (several trailing
/// lines collapse onto the same timestamp, especially after non-spoken tags like
/// `[pause]`). So instead of trusting each absolute start, we chain segment
/// **durations** from t=0, only jumping forward when a reported start runs ahead
/// of the running clock (which preserves genuine pauses). The schedule is
/// therefore always monotonic — it never runs backwards or dumps several lines
/// at once. Within a line, words are paced by cumulative character length.
/// `[audio tags]` are stripped out (they aren't spoken).
fn build_timeline(resp: &TextToDialogueWithTimestampsResponse, cast: &[Speaker]) -> Vec<Cue> {
    // Walk the segments in script order.
    let mut segments: Vec<&VoiceSegment> = resp.voice_segments.iter().collect();
    segments.sort_by_key(|s| s.dialogue_input_index);

    let mut cues = Vec::new();
    let mut clock = 0.0_f64;

    for seg in segments {
        let (name, color) = lookup(cast, &seg.voice_id);
        let words: Vec<String> = strip_tags(&resp.segment_text(seg))
            .split_whitespace()
            .map(String::from)
            .collect();
        if words.is_empty() {
            continue;
        }

        // Honour the reported start only if it's ahead of us (keeps real pauses),
        // and give each line a readable minimum span so words never bunch up.
        let line_start = seg.start_time_seconds.max(clock);
        let span = (seg.end_time_seconds - seg.start_time_seconds).max(words.len() as f64 * 0.09);
        let total: usize = words
            .iter()
            .map(|w| w.chars().count() + 1)
            .sum::<usize>()
            .max(1);

        let mut consumed = 0usize;
        for word in &words {
            let frac = consumed as f64 / total as f64;
            cues.push(Cue {
                start: line_start + frac * span,
                color,
                name,
                line: seg.dialogue_input_index,
                text: word.clone(),
            });
            consumed += word.chars().count() + 1;
        }
        clock = line_start + span;
    }

    cues
}

/// Remove `[audio tag]` spans from a line of dialogue, leaving the spoken text.
fn strip_tags(text: &str) -> String {
    let mut out = String::new();
    let mut depth = 0u32;
    for ch in text.chars() {
        match ch {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Resolve a voice id back to its display name and colour.
fn lookup(cast: &[Speaker], voice_id: &str) -> (&'static str, &'static str) {
    cast.iter()
        .find(|s| String::from(s.voice.clone()) == voice_id)
        .map(|s| (s.name, s.color))
        .unwrap_or(("…", DIM))
}

/// Sleep until `clock` reaches `target` seconds, accounting for time already elapsed.
async fn wait_until(clock: Instant, target: f64) {
    let target = Duration::from_secs_f64(target.max(0.0));
    let elapsed = clock.elapsed();
    if target > elapsed {
        tokio::time::sleep(target - elapsed).await;
    }
}

/// Build a karaoke `.lrc` file: one timestamped line per dialogue turn.
fn build_lrc(resp: &TextToDialogueWithTimestampsResponse, cast: &[Speaker]) -> String {
    let mut lrc = String::from("[ti:The Last Ninety Seconds]\n[ar:elevenlabs_rs]\n\n");
    for (seg, text) in resp.segments_with_text() {
        let (name, _) = lookup(cast, &seg.voice_id);
        lrc.push_str(&format!(
            "{}{}: {}\n",
            lrc_time(seg.start_time_seconds),
            name,
            strip_tags(&text)
        ));
    }
    lrc
}

/// `[mm:ss.xx]` for LRC files.
fn lrc_time(t: f64) -> String {
    let minutes = (t / 60.0).floor() as u64;
    let seconds = t - (minutes as f64) * 60.0;
    format!("[{:02}:{:05.2}]", minutes, seconds)
}

/// `mm:ss.s` for the terminal transcript.
fn timestamp(t: f64) -> String {
    let minutes = (t / 60.0).floor() as u64;
    let seconds = t - (minutes as f64) * 60.0;
    format!("{:02}:{:04.1}", minutes, seconds)
}
