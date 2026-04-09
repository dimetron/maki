use std::borrow::Cow;
use std::env;
use std::path::Path;
use std::time::{Duration, Instant};

use super::{RetryInfo, Status};

use crate::animation::spinner_frame;
use crate::theme;

use maki_providers::{ModelPricing, TokenUsage};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub(crate) fn format_tokens(n: u32) -> String {
    match n {
        0..1_000 => n.to_string(),
        1_000..1_000_000 => format!("{:.1}k", n as f64 / 1_000.0),
        _ => format!("{:.1}m", n as f64 / 1_000_000.0),
    }
}

const SPINNER_VERBS: &[&str] = &[
	"Accomplishing", "Actioning", "Actualizing", "Architecting",
	"Augmenting", "Avataring", "Baking", "Beaming",
	"Beboppin'", "Befuddling", "Billowing", "Bioforging",
	"Blanching", "Bloviating", "Boogieing", "Boondoggling",
	"Booping", "Bootstrapping", "Braindancing", "Breaching",
	"Brewing", "Bunning", "Burrowing", "Calculating",
	"Canoodling", "Caramelizing", "Cascading", "Catapulting",
	"Cerebrating", "Channeling", "Chipburning", "Choreographing",
	"Chroming", "Churning", "Ciphering", "Coalescing",
	"Cogitating", "Combobulating", "Compiling", "Composing",
	"Computing", "Concocting", "Considering", "Constructing",
	"Contemplating", "Cooking", "Coreslicing", "Cowboying",
	"Crafting", "Creating", "Crunching", "Crystallizing",
	"Cultivating", "Cyberdecking", "Darkpooling", "Datamining",
	"Datavaulting", "Deciphering", "Decompiling", "Decrypting",
	"Deliberating", "Deltasleeping", "Depixelating", "Dermatroding",
	"Determining", "Dilly-dallying", "Discombobulating", "Dissolving",
	"Doodling", "Downlinking", "Drifting", "Drizzling",
	"Ebbing", "Edgerunning", "Effecting", "Electrogliding",
	"Elucidating", "Embellishing", "Enchanting", "Encrypting",
	"Envisioning", "Evaporating", "Fermenting", "Fiddle-faddling",
	"Finagling", "Firewalling", "Flatcoding", "Flowing",
	"Flummoxing", "Fluttering", "Forging", "Forming",
	"Fragmenting", "Frolicking", "Frosting", "Gallivanting",
	"Galloping", "Gargoyleing", "Garnishing", "Generating",
	"Germinating", "Gesticulating", "Ghosting", "Glitching",
	"Gridwalking", "Grooving", "Gusting", "Hardwiring",
	"Harmonizing", "Hashing", "Hatching", "Herding",
	"Hexdumping", "Hologramming", "Honking", "Hotswapping",
	"Hullaballooing", "Hyperspacing", "Hyperthreading", "Ideating",
	"Imagining", "Improvising", "Incubating", "Inferring",
	"Infusing", "Interfacing", "Ionizing", "Iterating",
	"Jitterbugging", "Jockeying", "Julienning", "Kernelizing",
	"Kneading", "Leavening", "Levitating", "Linecooking",
	"Lollygagging", "Looping", "Manifesting", "Marinating",
	"Matrixing", "Meandering", "Megafluxing", "Meshing",
	"Metamorphosing", "Metaversing", "Mirrorshading", "Misting",
	"Moonwalking", "Morphing", "Moseying", "Mulling",
	"Mustering", "Musing", "Nanoweaving", "Nebulizing",
	"Neontracing", "Nesting", "Netrunning", "Neural-linking",
	"Neuromancing", "Noodling", "Nucleating", "Obfuscating",
	"Orbiting", "Orchestrating", "Osmosing", "Overclocking",
	"Overwatching", "Perambulating", "Percolating", "Perusing",
	"Philosophizing", "Photosynthesizing", "Pixeldrifting", "Pollinating",
	"Pondering", "Pontificating", "Pouncing", "Precipitating",
	"Prestidigitating", "Processing", "Proofing", "Propagating",
	"Puttering", "Puzzling", "Quantumizing", "Razzle-dazzling",
	"Razzmatazzing", "Recompiling", "Recombobulating", "Reflashing",
	"Reticulating", "Roosting", "Ruminating", "Sandboxing",
	"Scampering", "Schlepping", "Scurrying", "Seasoning",
	"Shadowcasting", "Shenaniganing", "Shimmying", "Simsense-loading",
	"Simstimming", "Simmering", "Skedaddling", "Sketching",
	"Slithering", "Smooshing", "Sock-hopping", "Soldering",
	"Spelunking", "Spinning", "Sprawling", "Sprouting",
	"Stewing", "Sublimating", "Subroutining", "Swirling",
	"Swooping", "Symbioting", "Synapse-firing", "Synthwaving",
	"Synthesizing", "Tempering", "Tessier-ashpooling", "Thinking",
	"Thundering", "Tinkering", "Tomfoolering", "Topsy-turvying",
	"Tracing", "Transfiguring", "Transmuting", "Trode-jockeying",
	"Tunneling", "Twisting", "Undulating", "Unfurling",
	"Unraveling", "Uplinking", "Vaporwaving", "Vibing",
	"Voodoo-boying", "Voxelizing", "Waddling", "Wandering",
	"Warping", "Wetware-syncing", "Whatchamacalliting", "Whirlpooling",
	"Whirring", "Whisking", "Wibbling", "Wintermuting",
	"Wireframing", "Working", "Wrangling", "Zesting",
	"Zigzagging", "Zone-tripping",
];

pub struct UsageStats<'a> {
    pub usage: &'a TokenUsage,
    pub global_usage: &'a TokenUsage,
    pub context_size: u32,
    pub pricing: &'a ModelPricing,
    pub context_window: u32,
    pub show_global: bool,
}

pub struct StatusBarContext<'a> {
    pub status: &'a Status,
    pub mode_label: Cow<'static, str>,
    pub mode_style: Style,
    pub model_id: &'a str,
    pub stats: UsageStats<'a>,
    pub auto_scroll: bool,
    pub chat_name: Option<&'a str>,
    pub retry_info: Option<&'a RetryInfo>,
    pub thinking_label: Option<Cow<'static, str>>,
}

const SPINNER_VERB_INTERVAL_MS: u128 = 3_000;

pub struct StatusBar {
    flash: Option<(String, Instant)>,
    started_at: Instant,
    cwd_branch: String,
    pub flash_duration: Duration,
    branch_update_rx: Option<flume::Receiver<()>>,
    spinner_verb_idx: usize,
    spinner_verb_at: Instant,
}

impl StatusBar {
    pub fn new(flash_duration: Duration) -> Self {
        let now = Instant::now();
        Self {
            flash: None,
            started_at: now,
            cwd_branch: cwd_branch_label(),
            flash_duration,
            branch_update_rx: spawn_branch_watcher(),
            spinner_verb_idx: (now.elapsed().as_millis() as usize) % SPINNER_VERBS.len(),
            spinner_verb_at: now,
        }
    }

    pub fn flash(&mut self, msg: String) {
        self.flash = Some((msg, Instant::now()));
    }

    #[cfg(test)]
    pub fn flash_text(&self) -> Option<&str> {
        self.flash.as_ref().map(|(s, _)| s.as_str())
    }

    pub fn refresh_cwd(&mut self) {
        self.cwd_branch = cwd_branch_label();
    }

    pub fn poll_branch_update(&mut self) {
        let Some(rx) = &self.branch_update_rx else {
            return;
        };
        if rx.try_iter().next().is_some() {
            self.cwd_branch = cwd_branch_label();
        }
    }

    pub fn tick_spinner(&mut self) {
        let elapsed = self.spinner_verb_at.elapsed().as_millis();
        if elapsed >= SPINNER_VERB_INTERVAL_MS {
            self.spinner_verb_idx = self.spinner_verb_idx.wrapping_add(1) % SPINNER_VERBS.len();
            self.spinner_verb_at = Instant::now();
        }
    }

    /// Advance the spinner verb immediately (e.g., after a thinking event)
    pub fn advance_spinner_verb(&mut self) {
        self.spinner_verb_idx = self.spinner_verb_idx.wrapping_add(1) % SPINNER_VERBS.len();
        self.spinner_verb_at = Instant::now();
    }

    pub fn clear_flash(&mut self) {
        self.flash = None;
    }

    pub fn clear_expired_hint(&mut self) {
        if self
            .flash
            .as_ref()
            .is_some_and(|(_, t)| t.elapsed() >= self.flash_duration)
        {
            self.flash = None;
        }
    }

    pub fn view(&self, frame: &mut Frame, area: Rect, ctx: &StatusBarContext) {
        let mut left_spans = Vec::new();

        if *ctx.status == Status::Streaming {
            let ch = spinner_frame(self.started_at.elapsed().as_millis());
            left_spans.push(Span::styled(format!(" {ch}"), theme::current().spinner));
            let verb = SPINNER_VERBS[self.spinner_verb_idx % SPINNER_VERBS.len()];
            left_spans.push(Span::styled(format!(" {verb}"), theme::current().spinner));
        }

        left_spans.push(Span::styled(format!(" {}", ctx.mode_label), ctx.mode_style));

        if let Some(name) = ctx.chat_name {
            left_spans.push(Span::styled(
                format!(" [{name}]"),
                theme::current().status_context,
            ));
        }

        if !ctx.auto_scroll {
            left_spans.push(Span::styled(
                " auto-scroll paused",
                theme::current().status_context,
            ));
        }

        if let Some(retry) = ctx.retry_info {
            let secs = retry
                .deadline
                .saturating_duration_since(Instant::now())
                .as_secs();
            left_spans.push(Span::styled(
                format!(" {}", retry.message),
                theme::current().status_retry_error,
            ));
            left_spans.push(Span::styled(
                format!(" · retrying in {secs}s (#{})", retry.attempt),
                theme::current().status_retry_info,
            ));
        }

        let mut right_spans = Vec::new();

        match ctx.status {
            Status::Error { message: e, .. } => {
                left_spans.push(Span::styled(format!(" {e}"), theme::current().error));
            }
            _ => {
                let pct = if ctx.stats.context_window > 0 {
                    (ctx.stats.context_size as f64 / ctx.stats.context_window as f64 * 100.0) as u32
                } else {
                    0
                };

                right_spans.push(Span::styled(
                    self.cwd_branch.clone(),
                    theme::current().status_context,
                ));
                right_spans.push(Span::raw("  "));
                right_spans.push(Span::styled(
                    ctx.model_id.to_string(),
                    theme::current().status_context,
                ));

                if let Some(ref label) = ctx.thinking_label {
                    right_spans.push(Span::styled(
                        format!(" [{label}]"),
                        theme::current().status_context,
                    ));
                }

                let rest_text = format!(
                    "  {} ({}%) ${:.3} ",
                    format_tokens(ctx.stats.context_size),
                    pct,
                    ctx.stats.usage.cost(ctx.stats.pricing),
                );
                right_spans.push(Span::styled(
                    rest_text,
                    Style::new().fg(theme::current().foreground),
                ));

                if ctx.stats.show_global {
                    let global_text = format!(
                        " \u{03a3}${:.3} ",
                        ctx.stats.global_usage.cost(ctx.stats.pricing),
                    );
                    right_spans.push(Span::styled(
                        global_text,
                        Style::new().fg(theme::current().foreground),
                    ));
                }
            }
        }

        if let Some((ref msg, _)) = self.flash {
            left_spans.push(Span::styled(
                format!(" {msg}"),
                theme::current().status_flash,
            ));
        }

        let [left_area, right_area] = Layout::horizontal([
            Constraint::Min(0),
            Constraint::Length(right_spans.iter().map(|s| s.width() as u16).sum()),
        ])
        .areas(area);

        frame.render_widget(Paragraph::new(Line::from(left_spans)), left_area);
        frame.render_widget(
            Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right),
            right_area,
        );
    }
}

fn collapse_home(path: &str) -> String {
    let Some(home) = dirs::home_dir() else {
        return path.to_string();
    };
    collapse_home_with(path, &home.to_string_lossy())
}

fn collapse_home_with(path: &str, home: &str) -> String {
    path.strip_prefix(home)
        .map(|rest| format!("~{rest}"))
        .unwrap_or_else(|| path.to_string())
}

fn cwd_branch_label() -> String {
    let cwd = env::current_dir()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| ".".into());
    let label = collapse_home(&cwd);
    match detect_branch(&cwd) {
        Some(branch) => format!("{label}:{branch}"),
        None => label,
    }
}

fn detect_branch(cwd: &str) -> Option<String> {
    let head = std::fs::read_to_string(find_git_dir(Path::new(cwd))?.join("HEAD")).ok()?;
    let head = head.trim();
    head.strip_prefix("ref: refs/heads/")
        .map(str::to_string)
        .or_else(|| Some(head.get(..7)?.to_string()))
}

fn find_git_dir(cwd: &Path) -> Option<std::path::PathBuf> {
    let mut dir = cwd;
    loop {
        let git = dir.join(".git");
        if git.is_dir() {
            return Some(git);
        }
        dir = dir.parent()?;
    }
}

fn spawn_branch_watcher() -> Option<flume::Receiver<()>> {
    use notify::{RecursiveMode, Watcher};

    let cwd = env::current_dir().ok()?;
    let git_dir = find_git_dir(&cwd)?;
    let (tx, rx) = flume::bounded(1);

    std::thread::spawn(move || {
        let Ok(mut watcher) = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
            if res.is_ok_and(|e| e.paths.iter().any(|p| p.ends_with("HEAD"))) {
                let _ = tx.try_send(());
            }
        }) else {
            return;
        };
        if watcher.watch(&git_dir, RecursiveMode::NonRecursive).is_ok() {
            std::thread::park();
        }
    });

    Some(rx)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use tempfile::TempDir;
    use test_case::test_case;

    #[test_case(999, "999")]
    #[test_case(1_000, "1.0k")]
    #[test_case(12_345, "12.3k")]
    #[test_case(999_999, "1000.0k")]
    #[test_case(1_000_000, "1.0m")]
    #[test_case(1_500_000, "1.5m")]
    fn format_tokens_display(input: u32, expected: &str) {
        assert_eq!(format_tokens(input), expected);
    }

    #[test_case("/home/user/projects/app", "/home/user", "~/projects/app" ; "inside_home")]
    #[test_case("/tmp/other", "/home/user", "/tmp/other"                  ; "outside_home")]
    #[test_case("/home/user", "/home/user", "~"                           ; "exact_home")]
    fn collapse_home_cases(path: &str, home: &str, expected: &str) {
        assert_eq!(collapse_home_with(path, home), expected);
    }

    fn tmp_with_head(content: Option<&str>) -> (TempDir, String) {
        let dir = TempDir::new().unwrap();
        if let Some(head) = content {
            let git = dir.path().join(".git");
            fs::create_dir(&git).unwrap();
            fs::write(git.join("HEAD"), head).unwrap();
        }
        let path = dir.path().to_string_lossy().into_owned();
        (dir, path)
    }

    #[test_case(Some("ref: refs/heads/feature/foo\n"), Some("feature/foo") ; "regular_ref")]
    #[test_case(Some("abc1234deadbeef\n"),            Some("abc1234")      ; "detached_head")]
    #[test_case(None,                                 None                 ; "no_git_dir")]
    fn detect_branch_cases(head: Option<&str>, expected: Option<&str>) {
        let (_dir, path) = tmp_with_head(head);
        assert_eq!(detect_branch(&path), expected.map(String::from));
    }

    #[test]
    fn detect_branch_from_subdirectory() {
        let (_dir, path) = tmp_with_head(Some("ref: refs/heads/main\n"));
        let sub = Path::new(&path).join("sub");
        fs::create_dir(&sub).unwrap();
        assert_eq!(
            detect_branch(&sub.to_string_lossy()),
            Some("main".to_string())
        );
    }

    #[test]
    fn clear_expired_hint_removes_stale_flash() {
        let mut bar = StatusBar::new(Duration::ZERO);
        bar.flash("Copied".into());
        bar.clear_expired_hint();
        assert!(bar.flash.is_none());
    }

    #[test]
    fn clear_flash_removes_flash() {
        let mut bar = StatusBar::new(Duration::from_secs(999));
        bar.flash("Copied".into());
        bar.clear_flash();
        assert!(bar.flash.is_none());
    }
}
