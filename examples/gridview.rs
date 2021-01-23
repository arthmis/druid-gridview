use druid_gridview::GridView;
use rand::distributions::{Distribution, Uniform};
use rand::{Rng, SeedableRng};
use std::sync::Arc;

use druid::{
    widget::{Container, Flex, List, MainAxisAlignment, Painter, Scroll, SizedBox},
    AppLauncher, Color, Data, Lens, RenderContext, UnitPoint, Widget, WidgetExt, WindowDesc,
};

fn main() {
    let data = {
        let mut data = Vec::new();
        let between = Uniform::from(0..=255);
        // let mut rng = rand::thread_rng();
        let mut rng = rand::rngs::SmallRng::from_entropy();
        // for _ in 0..1_000 {
        for _ in 0..10 {
            let color = Color::rgb8(
                between.sample(&mut rng),
                between.sample(&mut rng),
                between.sample(&mut rng),
            );
            data.push(color);
        }
        data
    };

    // let window = WindowDesc::new(list_ui);
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

fn list_ui() -> impl Widget<AppState> {
    let list = List::new(|| {
        let painter = Painter::new(|ctx, data: &Color, _env| {
            // let background_color = data.clone();
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
    .lens(AppState::colors);

    Scroll::new(list).align_horizontal(UnitPoint::CENTER)
}

fn grid_ui() -> impl Widget<AppState> {
    let grid = GridView::new(|| {
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
    // .with_minor_axis_count(13)
    .wrap()
    // .with_spacing(5.)
    .lens(AppState::colors);

    Scroll::new(grid)
        .vertical()
        .fix_width(450.)
        .fix_height(900.)
        .align_horizontal(UnitPoint::CENTER)
    // Flex::column()
    //     // .with_child(grid)
    //     .with_flex_child(grid, 1.0)
    //     .main_axis_alignment(MainAxisAlignment::Center)
    //     .fix_width(450.)
    //     .fix_height(900.)
    //     .align_horizontal(UnitPoint::CENTER)
}
