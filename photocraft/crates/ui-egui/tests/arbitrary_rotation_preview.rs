//! #1436: a size-changing Arbitrary rotation preview must remain visible on the real GPU canvas.
//! Synthetic blue pixels are measured outside the dialog, so its controls cannot pass the test.
//! Where the dialog covers one edge of the rotated image (it does on smaller or differently scaled
//! screens, such as the Windows CI runner), that edge is checked against the dialog instead, and the
//! position is proven from the edge that can be seen.

use egui_kittest::kittest::Queryable;
use photocraft_ui_egui::PhotocraftApp;
use photocraft_ui_egui::control::{ControlRequest, Outcome, handle};
use serde_json::{Value, json};
use std::sync::Arc;

type Harness = egui_kittest::Harness<'static, PhotocraftApp>;

fn harness() -> Option<Harness> {
    let built = std::panic::catch_unwind(|| {
        egui_kittest::Harness::builder().with_size(egui::vec2(1280.0, 800.0)).with_pixels_per_point(1.0).with_max_steps(64).wgpu().build_eframe(|cc| {
            PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
            let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), Default::default());
            if let Some(rs) = cc.wgpu_render_state.as_ref() {
                app.set_wgpu(rs.clone());
            }
            app
        })
    });
    match built {
        Ok(h) => Some(h),
        Err(_) => {
            eprintln!("skipping: no GPU adapter");
            None
        }
    }
}

fn control(h: &mut Harness, method: &str, params: Value) -> Value {
    let ctx = h.ctx.clone();
    let (req, _rx) = ControlRequest::new(method, params);
    let Outcome::Done(value) = handle(h.state_mut(), &ctx, &req) else { panic!("{method} did not finish") };
    assert_eq!(value["ok"], true, "{method}: {value}");
    h.run_steps(4);
    value["result"].clone()
}

fn setup(h: &mut Harness, width: u32, height: u32, zoom: f32, center: [f32; 2]) {
    while h.state().session.active().is_some() {
        control(h, "engine.execute", json!({"command":"file.close","params":{"discard":true}}));
    }
    control(h, "engine.execute", json!({"command":"prefs.set","params":{"path":"interface.language","value":"en"}}));
    control(h, "engine.execute", json!({"command":"file.new","params":{"width":width,"height":height,"background":"white"}}));
    control(h, "engine.execute", json!({"command":"edit.fill","params":{"color":"#2266dd"}}));
    control(h, "ui.set", json!({"zoom":zoom,"center":center}));
    assert!(h.state().perf.gpu, "the fixture must use the GPU canvas");
}

fn open(h: &mut Harness) -> u64 {
    control(h, "ui.menu.invoke", json!({"id":"image.rotation.arbitrary"}))["dialog"].as_u64().expect("rotation dialog")
}

fn angle(h: &mut Harness, id: u64, value: f64) {
    control(h, "ui.dialog.set", json!({"dialog":id,"field":"angle","value":value}));
}

/// Count blue canvas pixels and their vertical extent, excluding every modal's actual rectangle.
fn blue(h: &mut Harness, name: &str) -> (usize, u32, u32) {
    let (count, top, bottom, _) = blue_with_modals(h, name);
    (count, top, bottom)
}

/// [`blue`], plus the modal rectangles that were left out of the count.
fn blue_with_modals(h: &mut Harness, name: &str) -> (usize, u32, u32, Vec<egui::Rect>) {
    h.run_steps(3);
    let rect = h.state().last_canvas_rect.shrink(3.0);
    let modals: Vec<_> =
        h.state().ui.dialogs.iter().map(|d| h.ctx.memory(|m| m.area_rect(egui::Id::new(("dialog", d.id)))).expect("laid-out dialog").expand(4.0)).collect();
    let img = h.render().expect("render");
    if let Some(dir) = std::env::var_os("PHOTOCRAFT_ROTATION_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).expect("evidence directory");
        img.save(std::path::Path::new(&dir).join(format!("{name}.png"))).expect("save UI frame");
    }
    let (mut count, mut top, mut bottom) = (0, u32::MAX, 0);
    for y in rect.top().max(0.0) as u32..(rect.bottom() as u32).min(img.height()) {
        for x in rect.left().max(0.0) as u32..(rect.right() as u32).min(img.width()) {
            if modals.iter().any(|r| r.contains(egui::pos2(x as f32, y as f32))) {
                continue;
            }
            let p = img.get_pixel(x, y).0;
            if p[0] < 80 && (60..160).contains(&p[1]) && p[2] > 170 {
                count += 1;
                top = top.min(y);
                bottom = bottom.max(y);
            }
        }
    }
    (count, top, bottom, modals)
}

/// Whether row `y` of the canvas is hidden by a modal (checked across the canvas width, with a
/// little slack for anti-aliased edges).
fn row_hidden(modals: &[egui::Rect], y: f32) -> bool {
    modals.iter().any(|r| y >= r.top() - 3.0 && y <= r.bottom() + 3.0)
}

#[test]
fn size_changing_rotation_preview_is_visible_and_preserves_the_panned_center() {
    let _gpu = photocraft_testkit::gpu_lock();
    let Some(mut h) = harness() else { return };
    h.run_steps(4);
    for (width, height, zoom, center, degrees, factor) in [
        (400, 240, 1.0, [160.0, 90.0], 45.0, 1),
        (400, 240, 1.0, [160.0, 90.0], -45.0, 1),
        (400, 240, 1.0, [160.0, 90.0], 90.0, 1),
        (3000, 1800, 0.12, [1350.0, 825.0], 45.0, 2),
    ] {
        setup(&mut h, width, height, zoom, center);
        assert_eq!(photocraft_ui_egui::proxy::factor(&h.state().session.active().unwrap().doc), factor);
        let id = open(&mut h);
        let source = blue(&mut h, &format!("{width}x{height}-{degrees}-zero")).0;
        assert!(source > 5_000, "the unrotated blue image must be visible outside the dialog");
        angle(&mut h, id, degrees);
        h.get_by_label("Preview").click();
        h.run_steps(4);
        assert_eq!(h.state().ui.dialogs[0].fields["__preview"], false);
        assert!(blue(&mut h, &format!("{width}x{height}-{degrees}-off")).0 > 5_000, "Preview off shows the original image");
        h.get_by_label("Preview").click();
        h.run_steps(4);
        assert_eq!(h.state().ui.dialogs[0].fields["__preview"], true);
        let (count, top, bottom, modals) = blue_with_modals(&mut h, &format!("{width}x{height}-{degrees}-on"));
        assert!(count > 5_000, "{width}x{height}, {degrees}°, factor {factor}: rotated blue pixels disappeared ({count} remain)");
        let rect = photocraft_ui_egui::rulers::content_rect(h.state(), h.state().last_canvas_rect);
        let expected_y = rect.center().y + (height as f32 / 2.0 - center[1]) * zoom;
        // The rotated image's height on screen, from the rotation itself.
        let radians = (degrees as f32).to_radians();
        let half = (width as f32 * radians.sin().abs() + height as f32 * radians.cos().abs()) * zoom / 2.0;
        let (expected_top, expected_bottom) = (expected_y - half, expected_y + half);
        let detail = format!(
            "{width}x{height}, {degrees}°: expected blue rows {expected_top:.0}..{expected_bottom:.0} (center {expected_y}), got {top}..{bottom} ({count} px, canvas {rect:?}, dialogs {modals:?}, max texture side {})",
            h.ctx.input(|i| i.max_texture_side)
        );
        // Each edge is either where the rotation puts it, or hidden by a dialog that the visible
        // blue runs into. At least one edge must be visible, so the position is always proven.
        let edge = |actual: u32, expected: f32| (actual as f32 - expected).abs() < 3.0;
        let top_ok = edge(top, expected_top);
        let bottom_ok = edge(bottom, expected_bottom);
        let top_hidden = !top_ok && top as f32 > expected_top && row_hidden(&modals, expected_top) && row_hidden(&modals, top as f32 - 1.0);
        let bottom_hidden =
            !bottom_ok && (bottom as f32) < expected_bottom && row_hidden(&modals, expected_bottom) && row_hidden(&modals, bottom as f32 + 1.0);
        assert!(top_ok || bottom_ok, "preview moved the panned image center: {detail}");
        assert!(top_ok || top_hidden, "top edge is neither in place nor under a dialog: {detail}");
        assert!(bottom_ok || bottom_hidden, "bottom edge is neither in place nor under a dialog: {detail}");
        h.get_by_label("Cancel").click();
        h.run_steps(4);
    }
}

#[test]
fn preview_toggle_and_cancel_keep_the_document_while_ok_applies_rotation() {
    let _gpu = photocraft_testkit::gpu_lock();
    let Some(mut h) = harness() else { return };
    h.run_steps(4);
    setup(&mut h, 400, 240, 1.0, [200.0, 120.0]);
    let st = h.state().session.active().unwrap();
    let (doc, revision, history) = (st.doc.clone(), st.revision, st.history.entries());
    let id = open(&mut h);
    assert!(blue(&mut h, "control-zero-on").0 > 5_000);
    h.get_by_label("Preview").click();
    h.run_steps(4);
    assert_eq!(h.state().ui.dialogs[0].fields["__preview"], false);
    assert!(blue(&mut h, "control-zero-off").0 > 5_000);
    h.get_by_label("Preview").click();
    h.run_steps(4);
    assert_eq!(h.state().ui.dialogs[0].fields["__preview"], true);
    assert!(blue(&mut h, "control-zero-restored").0 > 5_000);
    h.get_by_label("Preview").click();
    h.run_steps(4);
    angle(&mut h, id, 45.0);
    h.get_by_label("Cancel").click();
    h.run_steps(4);
    let st = h.state().session.active().unwrap();
    assert!(Arc::ptr_eq(&st.doc, &doc), "Cancel must keep the committed document");
    assert_eq!(st.revision, revision);
    assert_eq!(st.history.entries(), history);
    assert!(blue(&mut h, "control-cancel").0 > 5_000);

    let id = open(&mut h);
    h.get_by_label("Preview").click();
    h.run_steps(4);
    angle(&mut h, id, 45.0);
    h.get_by_label("OK").click();
    h.run_steps(4);
    assert!(h.state().ui.dialogs.is_empty());
    let st = h.state().session.active().unwrap();
    assert_eq!((st.doc.size.width, st.doc.size.height), (453, 453));
    assert_eq!(st.history.entries().len(), history.len() + 1);
    assert!(st.revision > revision);
    assert_eq!(h.state().ui.views[0].center, [226.5, 226.5]);
    assert!(blue(&mut h, "control-ok").0 > 5_000, "the applied rotation must be visible");
}
