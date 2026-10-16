use fframes::{
    AudioMap, AudioTimestamp::*, AudioTrack, Color, Duration, FFramesContext, Frame, Scene, Scenes, Svgr, Video,
};
use std::collections::HashMap;

pub const WIDTH: usize = 960;
pub const HEIGHT: usize = 720;
pub const FRAMES: usize = 620;

/// One vector layer: fill colour and SVG path data, in reference-frame pixels.
pub struct Layer {
    pub fill: String,
    pub d: String,
}

fn read_layers(path: &std::path::Path) -> Vec<Layer> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .map(|(f, d)| Layer { fill: f.to_owned(), d: d.to_owned() })
        .collect()
}

/// Named vector assets in assets/vec/*.txt.
fn load_vectors() -> HashMap<String, Vec<Layer>> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/vec");
    let mut map = HashMap::new();
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) == Some("txt") {
            map.insert(p.file_stem().unwrap().to_string_lossy().into_owned(), read_layers(&p));
        }
    }
    map
}

/// Rotoscoped frames in assets/roto/f####.txt, one per reference frame (empty when missing).
fn load_roto() -> Vec<Vec<Layer>> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/roto");
    (0..FRAMES).map(|i| read_layers(&std::path::Path::new(dir).join(format!("f{i:04}.txt")))).collect()
}

#[derive(Debug)]
pub struct Shot {
    pub name: &'static str,
    pub start: usize,
    pub end: usize,
}

impl Scene for Shot {
    fn duration(&self) -> Duration<'_> { Duration::Frames(self.end - self.start) }
    fn name(&self) -> &'static str { self.name }
    fn render_frame<'a>(&'a self, _frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> { Svgr::empty() }
}

/// Shot boundaries, in reference frames (first frame of each shot).
pub const CUTS: [(&str, usize); 23] = [
    ("Sky", 0), ("Meadow", 1), ("Daisy", 40), ("Grid", 87), ("Flash", 102), ("Bubble", 105),
    ("Discs", 122), ("Sun", 165), ("Planet", 180), ("Swirl", 198), ("Table", 222),
    ("Dreamer", 251), ("Clock", 290), ("Rings", 308), ("Beans", 330), ("Refresh", 353),
    ("CupIntro", 369), ("Blush", 399), ("Spark", 447), ("Latte", 450), ("Breakfast", 463),
    ("Blur", 541), ("Thanks", 548),
];

pub struct ClipVideo {
    pub vec: HashMap<String, Vec<Layer>>,
    pub roto: Vec<Vec<Layer>>,
    pub shots: Vec<Shot>,
}

impl ClipVideo {
    pub fn new() -> Self {
        let shots = CUTS
            .iter()
            .enumerate()
            .map(|(i, &(name, start))| Shot { name, start, end: CUTS.get(i + 1).map_or(FRAMES, |c| c.1) })
            .collect();
        Self { vec: load_vectors(), roto: load_roto(), shots }
    }

    fn paths<'a>(&'a self, layers: &'a [Layer]) -> Vec<Svgr<'a>> {
        layers
            .iter()
            .map(|l| fframes::svgr!(<path d={l.d.as_str()} fill={l.fill.as_str()} fill-rule="evenodd" />))
            .collect()
    }

    /// The rotoscoped vector frame for reference frame `f` (paths are in half pixels).
    fn roto_frame(&self, f: usize) -> Svgr<'_> {
        let paths: Svgr = self.roto.get(f).map(|l| self.paths(l).into()).unwrap_or_default();
        fframes::svgr!(<g transform="scale(0.5)">{paths}</g>)
    }
}

impl Default for ClipVideo {
    fn default() -> Self { Self::new() }
}

impl Video for ClipVideo {
    const FPS: usize = 30;
    const WIDTH: usize = WIDTH;
    const HEIGHT: usize = HEIGHT;
    const BACKGROUND_COLOR: Color = Color::WHITE;
    fn duration(&self) -> Duration<'_> { Duration::Frames(FRAMES) }
    fn audio(&self) -> AudioMap<'_> { AudioMap::from([AudioTrack::new("soundtrack.wav", Second(0.0)..Eof)]) }
    fn define_scenes(&self) -> Scenes<'_> {
        Scenes::from(self.shots.iter().map(|s| s as &dyn Scene).collect::<Vec<_>>())
    }
    fn render_frame<'a>(&'a self, frame: Frame, _ctx: &FFramesContext<'a, '_>) -> Svgr<'a> {
        let f = frame.global_index;
        let body = self.roto_frame(f);
        fframes::svgr!(<svg xmlns="http://www.w3.org/2000/svg" width="960" height="720">
            {body}
        </svg>)
    }
}
