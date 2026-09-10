//! L'editeur : le cadre qui tient les editeurs dedies.
//!
//! `cargo run -p raster-editor -- <dossier du projet>`

use raster_core::ActorId;
use raster_core::asset::Project;
use raster_core::reflect::{Reflect, TypeRegistry, Value};
use raster_editor::Session;
use raster_editor::browser::{Browser, Kind};
use raster_editor::commands::{Despawn, Editing, MoveActors, SetField, Spawn};
use raster_editor::inspector::{self, Editor};
use raster_editor::keys::{self, Shortcuts};
use raster_editor::viewport::Viewport;
use raster_editor::{Dock, History, Node, Placement};
use raster_input::Input;
use raster_math::{Rect, Vec2};
use raster_render::{App, Camera, Colour, Gpu, RenderTarget, SpriteBatch, Texture, WindowConfig};
use raster_ui::layout::{self, Axis};
use raster_ui::{Align, Id, Keys, Painter, Pointer, Theme, Ui, widgets};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 800;
const WHITE: usize = 0;

/// Un acteur de demonstration, le temps que les scenes se chargent.
#[derive(Reflect, Default, Debug)]
struct Prop {
    position: Vec2,
    #[property(min = 0.0, max = 4.0)]
    scale: f32,
    solid: bool,
    name: String,
}

struct EditorApp {
    project: Option<Project>,
    browser: Browser,
    /// Le monde et son registre : ce que les commandes modifient.
    editing: Editing,
    dock: Dock,
    viewport: Viewport,
    history: History,
    ui: Ui,
    batch: Option<SpriteBatch>,
    target: Option<RenderTarget>,
    textures: Vec<Texture>,
    camera: Camera,
    pointer: Pointer,
    keys: Keys,
    window: Vec2,
    search: String,
    /// La derniere chose que l'editeur a faite, montree dans la console.
    status: String,
    /// La scene ouverte, relative au projet.
    scene: Option<String>,
    /// Si les liaisons de l'editeur ont ete posees.
    bound: bool,
    quit: bool,
}

impl EditorApp {
    fn new(project: Option<Project>) -> Self {
        let mut browser = Browser::new();
        if let Some(p) = &project
            && let Err(e) = browser.scan(p)
        {
            eprintln!("le projet n'a pas pu etre lu : {e}");
        }

        let mut registry = TypeRegistry::new();
        registry.register::<Prop>();
        let mut editing = Editing::new(registry);

        // Deux acteurs pour avoir quelque chose a selectionner.
        editing.world.spawn(Prop {
            position: Vec2::new(32.0, 32.0),
            scale: 1.0,
            name: "mur".to_owned(),
            solid: true,
        });
        editing.world.spawn(Prop {
            position: Vec2::new(96.0, 64.0),
            scale: 1.0,
            name: "caisse".to_owned(),
            solid: false,
        });

        // La session du projet, ou la disposition par defaut.
        let session = project.as_ref().map_or_else(
            || Session::new(Dock::new(shell_layout())),
            |p| Session::load(p.root(), Dock::new(shell_layout())),
        );

        let mut viewport = Viewport::new();
        viewport.snap = session.snap;
        viewport.show_grid = session.show_grid;
        viewport.zoom = session.zoom;

        Self {
            project,
            browser,
            editing,
            dock: session.dock,
            viewport,
            history: History::new(),
            ui: Ui::new(Theme::dark()),
            batch: None,
            target: None,
            textures: Vec::new(),
            camera: Camera::new(WIDTH, HEIGHT),
            pointer: Pointer::default(),
            keys: Keys::default(),
            window: Vec2::new(WIDTH as f32, HEIGHT as f32),
            search: String::new(),
            status: "pret".to_owned(),
            scene: session.scene,
            bound: false,
            quit: false,
        }
    }

    /// Ce que les raccourcis declenchent.
    fn apply_shortcuts(&mut self, shortcuts: Shortcuts) {
        if shortcuts.quit {
            self.save_session();
            self.quit = true;
        }

        if shortcuts.undo {
            self.history.undo(&mut self.editing);
            self.drop_dead_selection();
        }
        if shortcuts.redo {
            self.history.redo(&mut self.editing);
            self.drop_dead_selection();
        }

        if shortcuts.delete {
            for actor in self.viewport.selection().to_vec() {
                self.history
                    .push(Box::new(Despawn::new(actor)), &mut self.editing);
            }
            self.viewport.clear_selection();
        }

        if shortcuts.duplicate {
            self.duplicate_selection();
        }

        if shortcuts.save {
            self.status = match save_scene(self) {
                Ok(path) => format!("enregistre : {}", path.display()),
                Err(e) => e,
            };
        }

        if shortcuts.open {
            self.status = match load_scene(self) {
                Ok(n) => format!("{n} acteur(s) charge(s)"),
                Err(e) => e,
            };
        }

        if shortcuts.toggle_snap {
            self.viewport.snap = !self.viewport.snap;
        }
        if shortcuts.toggle_grid {
            self.viewport.show_grid = !self.viewport.show_grid;
        }
    }

    /// Une selection qui survit a une suppression pointerait dans le vide.
    fn drop_dead_selection(&mut self) {
        let world = &self.editing.world;
        self.viewport.retain_selection(|id| world.contains(id));
    }

    /// Duplique la selection, decalee d'une tuile pour qu'elle se voie.
    fn duplicate_selection(&mut self) {
        let copies: Vec<Vec2> = self
            .viewport
            .selection()
            .iter()
            .filter_map(|id| self.editing.world.get::<Prop>(*id))
            .map(|prop| prop.position + Vec2::splat(self.viewport.grid))
            .collect();

        for at in copies {
            self.history
                .push(Box::new(Spawn::new("Prop", at)), &mut self.editing);
        }
    }

    /// Enregistre la disposition et les reglages, s'il y a un projet.
    fn save_session(&self) {
        let Some(project) = &self.project else {
            return;
        };

        let mut session = Session::new(self.dock.clone());
        session.snap = self.viewport.snap;
        session.show_grid = self.viewport.show_grid;
        session.zoom = self.viewport.zoom;
        session.scene = self.scene.clone();

        if let Err(e) = session.save(project.root()) {
            eprintln!("la session n'a pas pu etre enregistree : {e}");
        }
    }

    /// Les acteurs de la scene, avec leur rectangle dans le monde.
    fn actors(&self) -> Vec<(ActorId, Rect, String)> {
        self.editing
            .world
            .iter::<Prop>()
            .map(|(id, prop)| {
                let size = 16.0 * prop.scale.max(0.25);
                (
                    id,
                    Rect::new(prop.position.x, prop.position.y, size, size),
                    prop.name.clone(),
                )
            })
            .collect()
    }
}

/// La disposition par defaut : assets a gauche, inspecteur a droite, console
/// en bas, le viewport au milieu.
fn shell_layout() -> Node {
    Node::split(
        Axis::Horizontal,
        0.18,
        Node::tabs(&["Assets"]),
        Node::split(
            Axis::Horizontal,
            0.78,
            Node::split(
                Axis::Vertical,
                0.76,
                Node::tabs(&["Scene"]),
                Node::tabs(&["Console"]),
            ),
            Node::tabs(&["Inspecteur"]),
        ),
    )
}

impl App for EditorApp {
    fn init(&mut self, gpu: &mut Gpu) {
        let batch = SpriteBatch::new(gpu);
        let layout = batch.texture_layout();
        self.textures = vec![Texture::white(gpu, layout)];
        self.batch = Some(batch);
        self.target = Some(RenderTarget::new(gpu, WIDTH, HEIGHT));
        self.camera.position = Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
    }

    fn update(&mut self, input: &mut Input, _time: &raster_core::Time) {
        // Les liaisons de l'editeur, posees a la premiere frame : `init` ne
        // recoit pas l'entree, donc c'est le premier endroit ou on peut.
        if !self.bound {
            *input.bindings_mut() = keys::bindings();
            self.bound = true;
        }

        let at = self
            .camera
            .screen_to_world(input.mouse_position(), self.window);
        self.pointer = Pointer::from_input(input, at);

        self.keys = Keys {
            confirm: input.pressed(&raster_input::Action::new("editor.confirm")),
            backspace: false,
            ..Keys::default()
        };

        let shortcuts = keys::read(input);
        self.apply_shortcuts(shortcuts);
    }

    fn resized(&mut self, width: u32, height: u32) {
        self.window = Vec2::new(width as f32, height as f32);
    }

    fn render(&mut self, gpu: &mut Gpu) {
        // Le batcher sort de `self` le temps du dessin : les panneaux ont
        // besoin de l'application entiere, et l'emprunter deux fois est refuse.
        // Il y retourne intact a la fin, sans etre reconstruit.
        let (Some(mut batch), Some(target)) = (self.batch.take(), self.target.take()) else {
            return;
        };
        let Some(mut frame) = gpu.begin_frame() else {
            self.batch = Some(batch);
            self.target = Some(target);
            return;
        };

        let theme = *self.ui.theme();
        target.clear(&mut frame, to_color(theme.background));

        let painter = Painter::new(WHITE);
        self.ui.begin(self.pointer, self.keys);

        let screen = Rect::new(0.0, 0.0, WIDTH as f32, HEIGHT as f32);
        for place in self.dock.layout(screen) {
            draw_panel(self, &mut batch, &painter, &place);
        }

        self.ui.end();

        batch.flush_into(gpu, &mut frame, target.view(), self.camera, &self.textures);
        let (w, h) = gpu.size();
        target.present(&mut frame, Vec2::new(w as f32, h as f32));
        gpu.end_frame(frame);

        self.batch = Some(batch);
        self.target = Some(target);
    }

    fn should_exit(&self) -> bool {
        self.quit
    }
}

fn draw_panel(app: &mut EditorApp, batch: &mut SpriteBatch, painter: &Painter, place: &Placement) {
    let theme = *app.ui.theme();

    // Le titre du panneau, puis son contenu en dessous.
    let header = Rect::new(
        place.area.position.x,
        place.area.position.y,
        place.area.size.x,
        16.0,
    );
    painter.rect(batch, place.area, theme.surface);
    painter.outline(batch, place.area, theme.border);
    painter.rect(batch, header, theme.pressed);
    painter.text(
        batch,
        Vec2::new(header.position.x + 4.0, header.position.y + 4.0),
        &place.panel,
        Align::Left,
        theme.muted,
    );

    let body = layout::inset(
        Rect::new(
            place.area.position.x,
            place.area.position.y + 16.0,
            place.area.size.x,
            (place.area.size.y - 16.0).max(0.0),
        ),
        4.0,
    );

    match place.panel.as_str() {
        "Assets" => draw_assets(app, batch, painter, body),
        "Scene" => draw_scene(app, batch, painter, body),
        "Inspecteur" => draw_inspector(app, batch, painter, body),
        _ => draw_console(app, batch, painter, body),
    }
}

fn draw_assets(app: &mut EditorApp, batch: &mut SpriteBatch, painter: &Painter, area: Rect) {
    let theme = *app.ui.theme();
    let rows = layout::stack(
        area,
        Axis::Vertical,
        4.0,
        &[layout::Size::Fixed(16.0), layout::Size::Grow(1.0)],
    );

    if widgets::text_field(
        &mut app.ui,
        batch,
        painter,
        Id::new("recherche"),
        rows[0],
        &mut app.search,
    ) {
        app.browser.search(&app.search);
    }

    let entries: Vec<(String, Kind, usize)> = app
        .browser
        .visible()
        .iter()
        .map(|e| (e.name.clone(), e.kind, e.depth))
        .collect();

    let content = entries.len() as f32 * theme.row_height;
    let (inner, clip) = widgets::scroll_area(
        &mut app.ui,
        batch,
        painter,
        Id::new("arbre"),
        rows[1],
        content,
    );

    for (i, (name, kind, depth)) in entries.iter().enumerate() {
        let y = inner.position.y + i as f32 * theme.row_height;
        let colour = match kind {
            Kind::Folder => theme.muted,
            Kind::Sprite => theme.accent,
            _ => theme.text,
        };
        painter.text(
            batch,
            Vec2::new(inner.position.x + *depth as f32 * 8.0, y + 2.0),
            name,
            Align::Left,
            colour,
        );
    }

    widgets::end_scroll(batch, clip);

    if app.browser.is_empty() {
        painter.text(
            batch,
            Vec2::new(area.position.x, area.position.y + 24.0),
            "Aucun projet ouvert",
            Align::Left,
            theme.muted,
        );
    }
}

fn draw_scene(app: &mut EditorApp, batch: &mut SpriteBatch, painter: &Painter, area: Rect) {
    let theme = *app.ui.theme();
    handle_scene_input(app, area);
    let previous = batch.push_clip(area);
    painter.rect(batch, area, theme.background);

    // La grille, sous les acteurs.
    if app.viewport.show_grid {
        let visible = app.viewport.visible(area);
        let step = app.viewport.grid;
        let mut x = (visible.position.x / step).floor() * step;

        while x < visible.position.x + visible.size.x {
            let sx = app.viewport.to_screen(area, Vec2::new(x, 0.0)).x;
            painter.rect(
                batch,
                Rect::new(sx, area.position.y, 1.0, area.size.y),
                theme.surface,
            );
            x += step;
        }

        let mut y = (visible.position.y / step).floor() * step;
        while y < visible.position.y + visible.size.y {
            let sy = app.viewport.to_screen(area, Vec2::new(0.0, y)).y;
            painter.rect(
                batch,
                Rect::new(area.position.x, sy, area.size.x, 1.0),
                theme.surface,
            );
            y += step;
        }
    }

    // Les acteurs, et leur selection.
    let zoom = app.viewport.zoom as f32;
    let actors = app.actors();

    for (id, bounds, _) in &actors {
        let at = app.viewport.to_screen(area, bounds.position);
        let rect = Rect::new(at.x, at.y, bounds.size.x * zoom, bounds.size.y * zoom);

        let response = app
            .ui
            .interact(Id::new("acteur").index(id.index() as usize), rect, true);
        if response.clicked {
            app.viewport.select(*id);
        }

        painter.rect(batch, rect, theme.accent);
        if app.viewport.is_selected(*id) {
            painter.outline(batch, rect, theme.focus);
        }
    }

    batch.pop_clip(previous);

    let etat = format!(
        "zoom x{}  grille {}  {} acteur(s)",
        app.viewport.zoom,
        if app.viewport.snap { "on" } else { "off" },
        actors.len()
    );
    painter.text(
        batch,
        Vec2::new(area.position.x + 2.0, area.position.y + area.size.y - 10.0),
        &etat,
        Align::Left,
        theme.muted,
    );
}

/// Ce que la souris fait dans le viewport : poser, choisir, deplacer.
fn handle_scene_input(app: &mut EditorApp, area: Rect) {
    let pointer = app.ui.pointer();
    if !area.contains(pointer.at) {
        return;
    }

    let world_at = app.viewport.to_world(area, pointer.at);

    // Molette : zoome autour du curseur.
    if pointer.wheel != 0.0 {
        app.viewport
            .zoom_at(area, pointer.at, if pointer.wheel < 0.0 { 1 } else { -1 });
    }

    // Un clic sur le vide pose un acteur ; sur un acteur, le choisit.
    if pointer.pressed {
        let touched = app
            .actors()
            .into_iter()
            .find(|(_, bounds, _)| bounds.contains(world_at))
            .map(|(id, _, _)| id);

        match touched {
            Some(id) => {
                app.viewport.select(id);
                app.viewport
                    .begin_drag(world_at, raster_editor::viewport::Mode::Move);
            }
            None => {
                let at = app.viewport.snapped(world_at);
                app.history
                    .push(Box::new(Spawn::new("Prop", at)), &mut app.editing);

                // Le dernier pose est celui qu'on vient de creer.
                if let Some((id, _)) = app.editing.world.iter::<Prop>().last() {
                    app.viewport.select(id);
                }
            }
        }
    }

    // Glisser deplace la selection, une entree d'annulation pour tout le geste.
    if pointer.down && app.viewport.dragging() {
        let delta = app.viewport.drag_to(world_at);
        if delta != Vec2::ZERO && !app.viewport.selection().is_empty() {
            let actors = app.viewport.selection().to_vec();
            app.history
                .push(Box::new(MoveActors::new(actors, delta)), &mut app.editing);
        }
    }

    if pointer.released && app.viewport.dragging() {
        app.viewport.end_drag();
        // Le geste est fini : le suivant ne doit pas s'y coller.
        app.history.break_merge();
    }
}

fn draw_inspector(app: &mut EditorApp, batch: &mut SpriteBatch, painter: &Painter, area: Rect) {
    let theme = *app.ui.theme();

    let Some(selected) = app.viewport.only_selected() else {
        painter.text(
            batch,
            area.position,
            "Rien de selectionne",
            Align::Left,
            theme.muted,
        );
        return;
    };

    let Some(prop) = app.editing.world.get::<Prop>(selected) else {
        return;
    };

    let rows = inspector::rows(prop);
    let mut y = area.position.y;

    for row in &rows {
        painter.text(
            batch,
            Vec2::new(area.position.x, y + 4.0),
            row.label,
            Align::Left,
            theme.muted,
        );

        let field = Rect::new(
            area.position.x + 80.0,
            y,
            (area.size.x - 84.0).max(20.0),
            14.0,
        );

        // Chaque type de champ a son editeur, choisi par la reflexion.
        match &row.editor {
            Editor::Checkbox => {
                let mut on = matches!(row.value, Value::Bool(true));
                if widgets::checkbox(
                    &mut app.ui,
                    batch,
                    painter,
                    Id::new("insp").child(row.field),
                    field,
                    "",
                    &mut on,
                )
                .clicked
                {
                    app.history.push(
                        Box::new(SetField::new(selected, row.field, Value::Bool(on))),
                        &mut app.editing,
                    );
                }
            }
            Editor::Slider { min, max } => {
                let mut value = as_f32(&row.value);
                if widgets::slider(
                    &mut app.ui,
                    batch,
                    painter,
                    Id::new("insp").child(row.field),
                    field,
                    (*min, *max),
                    &mut value,
                ) {
                    app.history.push(
                        Box::new(SetField::new(
                            selected,
                            row.field,
                            Value::Float(f64::from(value)),
                        )),
                        &mut app.editing,
                    );
                }
            }
            Editor::Vector2 => {
                let v = match row.value {
                    Value::Vec2(v) => v,
                    _ => Vec2::ZERO,
                };
                let halves = layout::stack(
                    field,
                    Axis::Horizontal,
                    4.0,
                    &[layout::Size::Grow(1.0), layout::Size::Grow(1.0)],
                );
                let mut x = v.x;
                let mut y_value = v.y;

                let a = widgets::drag_value(
                    &mut app.ui,
                    batch,
                    painter,
                    Id::new("insp").child(row.field).index(0),
                    halves[0],
                    &mut x,
                    1.0,
                );
                let b = widgets::drag_value(
                    &mut app.ui,
                    batch,
                    painter,
                    Id::new("insp").child(row.field).index(1),
                    halves[1],
                    &mut y_value,
                    1.0,
                );

                if a || b {
                    app.history.push(
                        Box::new(SetField::new(
                            selected,
                            row.field,
                            Value::Vec2(Vec2::new(x, y_value)),
                        )),
                        &mut app.editing,
                    );
                }
            }
            Editor::Text => {
                let texte = match &row.value {
                    Value::Str(s) => s.clone(),
                    other => format!("{other}"),
                };
                painter.text(
                    batch,
                    Vec2::new(field.position.x, field.position.y + 3.0),
                    &texte,
                    Align::Left,
                    theme.text,
                );
            }
            _ => {
                painter.text(
                    batch,
                    Vec2::new(field.position.x, field.position.y + 3.0),
                    &format!("{}", row.value),
                    Align::Left,
                    theme.disabled,
                );
            }
        }

        y += 18.0;
    }
}

fn draw_console(app: &mut EditorApp, batch: &mut SpriteBatch, painter: &Painter, area: Rect) {
    let theme = *app.ui.theme();
    let projet = app
        .project
        .as_ref()
        .map_or_else(|| "aucun".to_owned(), |p| p.root().display().to_string());

    let lignes = [
        format!("projet : {projet}"),
        format!("{} assets", app.browser.len()),
        format!(
            "historique : {} entree(s){}",
            app.history.depth(),
            if app.history.is_dirty() { " *" } else { "" }
        ),
        app.status.clone(),
        "Echap pour quitter".to_owned(),
    ];

    for (i, ligne) in lignes.iter().enumerate() {
        painter.text(
            batch,
            Vec2::new(area.position.x, area.position.y + i as f32 * 10.0),
            ligne,
            Align::Left,
            theme.muted,
        );
    }
}

/// Recharge la scene depuis le disque, remplacant ce qui est ouvert.
///
/// L'historique est vide : les commandes d'avant designent des acteurs qui
/// n'existent plus, et annuler apres un chargement n'a pas de sens.
fn load_scene(app: &mut EditorApp) -> Result<usize, String> {
    let project = app.project.as_ref().ok_or("aucun projet ouvert")?;
    let path = project.root().join("scene.scene.toml");

    let scene = raster_core::Scene::load(&path, &app.editing.registry)
        .map_err(|e| format!("le chargement a echoue : {e}"))?;

    app.editing.world.clear();
    app.viewport.clear_selection();

    let poses = scene
        .spawn_into(&mut app.editing.world, &app.editing.registry)
        .map_err(|e| format!("la scene n'a pas pu etre posee : {e}"))?;

    app.history.clear();
    app.scene = Some("scene.scene.toml".to_owned());
    Ok(poses.len())
}

/// Ecrit la scene sous forme de fichier, celui que le runtime sait charger.
fn save_scene(app: &mut EditorApp) -> Result<std::path::PathBuf, String> {
    let project = app.project.as_ref().ok_or("aucun projet ouvert")?;

    let mut scene = raster_core::Scene::new("scene");
    for (_, prop) in app.editing.world.iter::<Prop>() {
        scene.add(prop);
    }

    let path = project.root().join("scene.scene.toml");
    scene
        .save(&path)
        .map_err(|e| format!("l'enregistrement a echoue : {e}"))?;

    app.history.mark_saved();
    app.scene = Some("scene.scene.toml".to_owned());
    Ok(path)
}

fn as_f32(value: &Value) -> f32 {
    match value {
        Value::Float(v) => *v as f32,
        Value::Int(v) => *v as f32,
        _ => 0.0,
    }
}

fn to_color(colour: Colour) -> raster_render::Color {
    let [r, g, b, a] = colour.to_array();
    raster_render::Color {
        r: f64::from(r),
        g: f64::from(g),
        b: f64::from(b),
        a: f64::from(a),
    }
}

fn main() {
    // Le projet vient de la ligne de commande, ou du dossier courant.
    let project = std::env::args()
        .nth(1)
        .map(Project::new)
        .or_else(|| Project::discover(std::env::current_dir().unwrap_or_default()));

    if project.is_none() {
        eprintln!("aucun projet trouve : passez un dossier, ou lancez depuis un projet");
    }

    let config = WindowConfig {
        title: "Raster".to_owned(),
        width: WIDTH,
        height: HEIGHT,
        resizable: true,
    };

    if let Err(e) = raster_render::run(config, EditorApp::new(project)) {
        eprintln!("raster-editor: {e}");
    }
}
