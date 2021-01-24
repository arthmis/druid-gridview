use druid_gridview::GridView;
use rand::distributions::{Distribution, Uniform};
use rand::SeedableRng;
use std::sync::Arc;

use druid::{
    widget::{CrossAxisAlignment, Flex, MainAxisAlignment, Painter, Scroll, SizedBox},
    AppLauncher, Color, Data, Lens, RenderContext, Widget, WidgetExt, WindowDesc,
};

fn main() {
    let data = {
        let mut data = Vec::new();
        let between = Uniform::from(0..=255);
        let mut rng = rand::rngs::SmallRng::from_entropy();

        for _ in 0..50 {
            let color = Color::rgb8(
                between.sample(&mut rng),
                between.sample(&mut rng),
                between.sample(&mut rng),
            );
            data.push(color);
        }
        data
    };

    let window = WindowDesc::new(grid_ui);
    AppLauncher::with_window(window)
        .launch(AppState {
            colors: Arc::new(data),
        })
        .unwrap();
}

#[derive(Clone, Data, Lens)]
struct AppState {
    colors: Arc<Vec<Color>>,
}

fn grid_ui() -> impl Widget<AppState> {
    let left_vertical_grid = GridView::new(|| {
        let painter = Painter::new(|ctx, data: &Color, _env| {
            let rect = ctx.size().to_rect();
            ctx.stroke(rect, data, 0.);
            ctx.fill(rect, data);
        });

        SizedBox::empty()
            .width(150.)
            .height(150.)
            .background(painter)
    })
    .with_spacing(5.)
    .wrap()
    .lens(AppState::colors);

    let right_horizontal_grid = GridView::new(|| {
        let painter = Painter::new(|ctx, data: &Color, _env| {
            let rect = ctx.size().to_rect();
            ctx.stroke(rect, data, 0.);
            ctx.fill(rect, data);
        });

        SizedBox::empty()
            .width(150.)
            .height(150.)
            .background(painter)
    })
    .with_spacing(5.)
    .with_minor_axis_count(5)
    .wrap()
    .horizontal()
    .lens(AppState::colors);

    let left = Flex::row()
        .with_flex_spacer(0.1)
        .with_flex_child(
            Scroll::new(left_vertical_grid).vertical().expand_width(),
            0.8,
        )
        .with_flex_spacer(0.1)
        .main_axis_alignment(MainAxisAlignment::Center)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .center();

    let right = Flex::column()
        .with_flex_spacer(0.1)
        .with_flex_child(
            Scroll::new(right_horizontal_grid).horizontal().center(),
            0.8,
        )
        .with_flex_spacer(0.1)
        .center();

    Flex::row()
        .with_flex_child(left, 0.5)
        .with_spacer(5.)
        .with_flex_child(right, 0.5)
}
