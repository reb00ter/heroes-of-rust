/// build.rs — растеризует SVG-спрайты в PNG перед компиляцией игры.
/// Запускается автоматически при `cargo build`.
/// PNG пересоздаётся только если SVG новее или PNG отсутствует.
fn main() {
    render_svg(
        "assets/sprites/src/gold_pile.svg",
        "assets/sprites/gold_pile.png",
        2.0, // масштаб: 64×64 SVG → 128×128 PNG (Bevy сам масштабирует под тайл)
    );
}

fn render_svg(svg_path: &str, out_path: &str, scale: f32) {
    // Сообщаем Cargo: пересобирать только при изменении этого файла
    println!("cargo:rerun-if-changed={svg_path}");

    // Пропускаем, если PNG уже актуален
    if !needs_rebuild(svg_path, out_path) {
        return;
    }

    let svg_data = std::fs::read(svg_path)
        .unwrap_or_else(|e| panic!("[BUILD] Failed to read {svg_path}: {e}"));

    // Парсим SVG
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&svg_data, &opt)
        .unwrap_or_else(|e| panic!("[BUILD] Failed to parse {svg_path}: {e}"));

    // Размер пиксмапа с учётом масштаба
    let svg_size = tree.size();
    let w = (svg_size.width() * scale) as u32;
    let h = (svg_size.height() * scale) as u32;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)
        .unwrap_or_else(|| panic!("[BUILD] Failed to create pixmap {w}x{h}"));

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Создаём папку и сохраняем PNG
    if let Some(parent) = std::path::Path::new(out_path).parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|e| panic!("[BUILD] Failed to create directory {parent:?}: {e}"));
    }
    pixmap
        .save_png(out_path)
        .unwrap_or_else(|e| panic!("[BUILD] Failed to save {out_path}: {e}"));

    println!("cargo:warning=[BUILD] Sprite generated: {out_path} ({w}x{h}px)");
}

/// Возвращает true, если SVG новее PNG или PNG не существует.
fn needs_rebuild(src: &str, dst: &str) -> bool {
    let Ok(dst_meta) = std::fs::metadata(dst) else {
        return true;
    };
    let Ok(src_meta) = std::fs::metadata(src) else {
        return true; // SVG пропал — пересоберём и получим внятную ошибку
    };
    let src_time = src_meta
        .modified()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    let dst_time = dst_meta
        .modified()
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    src_time > dst_time
}
