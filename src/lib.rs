use std::{cmp::Ordering, fmt::Write, sync::Arc};

use druid::{
    widget::Axis, BoxConstraints, Data, Env, KeyOrValue, LifeCycle, Point, Rect, Size, Widget,
    WidgetPod,
};

enum MinorAxisCount {
    Wrap,
    Count(u64),
}
pub struct GridView<T> {
    closure: Box<dyn Fn() -> Box<dyn Widget<T>>>,
    children: Vec<WidgetPod<T, Box<dyn Widget<T>>>>,
    axis: Axis,
    spacing: KeyOrValue<f64>,
    minor_axis_count: MinorAxisCount,
}

impl<T: Data> GridView<T> {
    pub fn new<W: Widget<T> + 'static>(closure: impl Fn() -> W + 'static) -> Self {
        GridView {
            closure: Box::new(move || Box::new(closure())),
            children: Vec::new(),
            axis: Axis::Vertical,
            spacing: KeyOrValue::Concrete(0.),
            minor_axis_count: MinorAxisCount::Count(5),
        }
    }

    pub fn wrap(mut self) -> Self {
        self.minor_axis_count = MinorAxisCount::Wrap;
        self
    }

    pub fn with_minor_axis_count(mut self, count: u64) -> Self {
        self.minor_axis_count = MinorAxisCount::Count(count);
        self
    }

    pub fn with_spacing(mut self, spacing: impl Into<KeyOrValue<f64>>) -> Self {
        self.spacing = spacing.into();
        self
    }
    pub fn set_spacing(&mut self, spacing: impl Into<KeyOrValue<f64>>) -> &mut Self {
        self.spacing = spacing.into();
        self
    }

    fn update_child_count(&mut self, data: &impl GridIter<T>, _env: &Env) -> bool {
        let len = self.children.len();
        match len.cmp(&data.data_len()) {
            Ordering::Greater => self.children.truncate(data.data_len()),
            Ordering::Less => data.for_each(|_, i| {
                if i >= len {
                    let child = WidgetPod::new((self.closure)());
                    self.children.push(child);
                }
            }),
            Ordering::Equal => (),
        }
        len != data.data_len()
    }
}

pub trait GridIter<T>: Data {
    fn for_each(&self, cb: impl FnMut(&T, usize));

    fn for_each_mut(&mut self, cb: impl FnMut(&mut T, usize));

    fn data_len(&self) -> usize;

    fn child_data(&self) -> &T;
}

impl<T: Data> GridIter<T> for Arc<Vec<T>> {
    fn for_each(&self, mut cb: impl FnMut(&T, usize)) {
        for (i, item) in self.iter().enumerate() {
            cb(item, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut T, usize)) {
        let mut new_data = Vec::with_capacity(self.data_len());
        let mut any_changed = false;

        for (i, item) in self.iter().enumerate() {
            let mut d = item.to_owned();
            cb(&mut d, i);

            if !any_changed && !item.same(&d) {
                any_changed = true;
            }
            new_data.push(d);
        }

        if any_changed {
            *self = Arc::new(new_data);
        }
    }

    fn data_len(&self) -> usize {
        self.len()
    }

    fn child_data(&self) -> &T {
        self.last().unwrap()
    }
}

impl<C: Data, T: GridIter<C>> Widget<T> for GridView<C> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut T,
        env: &druid::Env,
    ) {
        let mut children = self.children.iter_mut();
        data.for_each_mut(|child_data, _| {
            if let Some(child) = children.next() {
                child.event(ctx, event, child_data, env);
            }
        })
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &T,
        env: &druid::Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            if self.update_child_count(data, env) {
                ctx.children_changed();
            }
        }

        let mut children = self.children.iter_mut();
        data.for_each(|child_data, _| {
            if let Some(child) = children.next() {
                child.lifecycle(ctx, event, child_data, env);
            }
        });
    }

    fn update(&mut self, ctx: &mut druid::UpdateCtx, _old_data: &T, data: &T, env: &druid::Env) {
        // we send update to children first, before adding or removing children;
        // this way we avoid sending update to newly added children, at the cost
        // of potentially updating children that are going to be removed.
        let mut children = self.children.iter_mut();
        data.for_each(|child_data, _| {
            if let Some(child) = children.next() {
                child.update(ctx, child_data, env);
            }
        });

        if self.update_child_count(data, env) {
            ctx.children_changed();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &druid::Env,
    ) -> druid::Size {
        let axis = self.axis;
        let spacing = self.spacing.resolve(env);
        let mut major_pos = 0.;
        let mut minor_pos = 0.;
        let mut paint_rect = Rect::ZERO;
        // let child_bc = constraints(axis, bc, 0., f64::INFINITY);
        // I don't know if this is the right way to go. I would assume a grid is
        // used in a Scroll and that would provide the infinite constraints if necessary
        // otherwise the scroll will be locked to an axis and provide concrete constraints
        // on that axis
        let child_bc = constraints(axis, bc, 0., bc.max().height);

        let minor_axis_count = match self.minor_axis_count {
            // this will assume grid is laid out vertically
            // one day this will account for both vertical and horizontal
            MinorAxisCount::Wrap => {
                let max_width = bc.max().width;
                let child_size = match self.children.last_mut() {
                    Some(child) => {
                        let size = child.layout(ctx, &child_bc, data.child_data(), env);
                        size
                    }
                    None => Size::ZERO,
                };
                dbg!(&child_size);
                if child_size == Size::ZERO {
                    // TODO: this should be zero, but i'm making it one to avoid divide by zero
                    1
                } else {
                    (max_width / child_size.width).floor() as usize
                }
            }
            MinorAxisCount::Count(count) => count as usize,
        };

        let mut children = self.children.iter_mut();

        data.for_each(|child_data, idx| {
            let child = match children.next() {
                Some(child) => child,
                None => return,
            };

            let child_size = child.layout(ctx, &child_bc, child_data, env);
            let child_pos: Point = axis.pack(major_pos, minor_pos).into();
            child.set_origin(ctx, child_data, env, child_pos);
            paint_rect = paint_rect.union(child.paint_rect());

            if (idx + 1) % minor_axis_count == 0 {
                // have to correct overshoot
                major_pos += axis.major(child_size) + spacing;
                minor_pos = 0.;
            } else {
                minor_pos += axis.minor(child_size) + spacing;
            }
            // have to correct overshoot
        });
        // let my_size = bc.constrain(Size::from(axis.pack(major_pos, minor_pos)));
        let my_size = bc.constrain(paint_rect.size());
        let insets = paint_rect - my_size.to_rect();
        ctx.set_paint_insets(insets);
        my_size
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &druid::Env) {
        let mut children = self.children.iter_mut();
        data.for_each(|child_data, _| {
            if let Some(child) = children.next() {
                child.paint(ctx, child_data, env);
            }
        });
    }
}
/// Generate constraints with new values on the major axis.
fn constraints(axis: Axis, bc: &BoxConstraints, min_major: f64, major: f64) -> BoxConstraints {
    match axis {
        Axis::Horizontal => BoxConstraints::new(
            Size::new(min_major, bc.min().height),
            Size::new(major, bc.max().height),
        ),
        Axis::Vertical => BoxConstraints::new(
            Size::new(bc.min().width, min_major),
            Size::new(bc.max().width, major),
        ),
    }
}
