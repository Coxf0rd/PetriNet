use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn rect_border_point(rect: Rect, dir: Vec2) -> Pos2 {
        let center = rect.center();
        let nx = if dir.x.abs() < f32::EPSILON {
            0.0
        } else {
            dir.x
        };
        let ny = if dir.y.abs() < f32::EPSILON {
            0.0
        } else {
            dir.y
        };
        let half_w = rect.width() * 0.5;
        let half_h = rect.height() * 0.5;
        let tx = if nx.abs() < f32::EPSILON {
            f32::INFINITY
        } else {
            half_w / nx.abs()
        };
        let ty = if ny.abs() < f32::EPSILON {
            f32::INFINITY
        } else {
            half_h / ny.abs()
        };
        let t = tx.min(ty);
        center + Vec2::new(nx * t, ny * t)
    }
}
