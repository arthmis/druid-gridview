//! A basic grid view widget.

use std::{cmp::Ordering, sync::Arc};

use druid::{
    widget::Axis, BoxConstraints, Data, Env, KeyOrValue, LifeCycle, Point, Rect, Size, Widget,
    WidgetPod,
};

/// A grid view widget for a variable size collection of items.
pub struct GridView<T> {
    closure: Box<dyn Fn() -> Box<dyn Widget<T>>>,
    children: Vec<WidgetPod<T, Box<dyn Widget<T>>>>,
    axis: Axis,
    vertical_spacing: KeyOrValue<f64>,
    horizontal_spacing: KeyOrValue<f64>,
    minor_axis_count: MinorAxisCount,
}

/// The number of elements found on the minor axis of the grid
enum MinorAxisCount {
    /// If this is wrap, the grid determines the max amount of items per
    /// minor axis. Wrap assumes the grid items are equal in size.
    Wrap,
    /// A user specified number of elements on minor axis. Can overflow
    /// the container if the count * size of grid items is larger than container
    Count(u64), // this should probably take a KeyOrValue<u64> instead
}

impl<T: Data> GridView<T> {
    /// Create a new grid view widget. The closure will be called when a new item needs
    /// to be constructed.
    ///
    /// Defaults to a vertical layout, 0 spacing between grid items and 5 items on the
    /// minor axis
    pub fn new<W: Widget<T> + 'static>(closure: impl Fn() -> W + 'static) -> Self {
        GridView {
            closure: Box::new(move || Box::new(closure())),
            children: Vec::new(),
            axis: Axis::Vertical,
            vertical_spacing: KeyOrValue::Concrete(0.),
            horizontal_spacing: KeyOrValue::Concrete(0.),
            minor_axis_count: MinorAxisCount::Count(5),
        }
    }

    // Sets the widget to display horizontally.
    pub fn horizontal(mut self) -> Self {
        self.axis = Axis::Horizontal;
        self
    }

    /// This will allow the grid to automatically determine how many items
    /// can be laid out on the minor axis before wrapping.
    ///
    /// If this is set along with [`with_minor_axis_count`], wrap will take priority.
    pub fn wrap(mut self) -> Self {
        self.minor_axis_count = MinorAxisCount::Wrap;
        self
    }

    /// Builder style method that sets how many elements will be laid out on the
    /// minor axis before the grid wraps around to the next row/column.
    ///
    /// If the amount of items * size of items is larger than the container, this will
    /// overflow the container. Use [`wrap`] to automatically wrap grid items.
    pub fn with_minor_axis_count(mut self, count: u64) -> Self {
        self.minor_axis_count = MinorAxisCount::Count(count);
        self
    }

    /// Sets how many elements will be laid out on the minor axis before the grid
    /// wraps around to the next row/column.
    ///
    /// If the amount of items * size of items is larger than the container, this will
    /// overflow the container. Use [`wrap`] to automatically wrap grid items.
    pub fn set_minor_axis_count(&mut self, count: u64) -> &mut Self {
        self.minor_axis_count = MinorAxisCount::Count(count);
        self
    }

    /// Builder style method that sets the vertical and horizontal spacing
    /// between elements to the same value.
    pub fn with_spacing(mut self, spacing: impl Into<KeyOrValue<f64>>) -> Self {
        let spacing = spacing.into();
        self.vertical_spacing = spacing.clone();
        self.horizontal_spacing = spacing;
        self
    }

    /// Sets the vertical and horizontal between elements to the same value.
    pub fn set_spacing(&mut self, spacing: impl Into<KeyOrValue<f64>>) -> &mut Self {
        let spacing = spacing.into();
        self.vertical_spacing = spacing.clone();
        self.horizontal_spacing = spacing;
        self
    }

    /// Builder style method that sets the spacing between elements vertically.
    pub fn with_vertical_spacing(mut self, spacing: impl Into<KeyOrValue<f64>>) -> Self {
        self.vertical_spacing = spacing.into();
        self
    }

    /// Sets the spacing between elements vertically.
    pub fn set_vertical_spacing(mut self, spacing: impl Into<KeyOrValue<f64>>) -> Self {
        self.vertical_spacing = spacing.into();
        self
    }

    /// Builder style method that sets the spacing between elements horizontally.
    pub fn with_horizontal_spacing(mut self, spacing: impl Into<KeyOrValue<f64>>) -> Self {
        self.horizontal_spacing = spacing.into();
        self
    }

    /// Sets the spacing between elements horizontally.
    pub fn set_horizontal_spacing(mut self, spacing: impl Into<KeyOrValue<f64>>) -> Self {
        self.horizontal_spacing = spacing.into();
        self
    }

    /// When the widget is created or the data changes, create or remove children as needed
    ///
    /// Returns `true` if children were added or removed.
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

/// This iterator enables writing GridView widget for any `Data`.
pub trait GridIter<T>: Data {
    fn for_each(&self, cb: impl FnMut(&T, usize));

    fn for_each_mut(&mut self, cb: impl FnMut(&mut T, usize));

    fn data_len(&self) -> usize;

    fn child_data(&self) -> Option<&T>;

    fn row(&self, cb: impl FnMut(&T, usize), row_len: usize);
    fn row_mut(&mut self, cb: impl FnMut(&mut T, usize), row_len: usize);
}

impl<T: Data> GridIter<T> for Arc<Vec<T>> {
    fn row(&self, mut cb: impl FnMut(&T, usize), row_len: usize) {
        let chunks_len = row_len;
        for (i, row) in self.chunks(chunks_len).enumerate() {
            for (j, item) in row.iter().enumerate() {
                cb(item, i * chunks_len + j)
            }
        }
    }
    fn row_mut(&mut self, mut cb: impl FnMut(&mut T, usize), row_len: usize) {
        let chunks_len = row_len;
        let mut new_data = Vec::with_capacity(self.data_len());
        let mut any_changed = false;

        for (i, row) in self.chunks(chunks_len).enumerate() {
            for (j, item) in row.iter().enumerate() {
                let mut d = item.to_owned();
                cb(&mut d, i * chunks_len + j);

                if !any_changed && !item.same(&d) {
                    any_changed = true;
                }
                new_data.push(d);
            }
        }

        if any_changed {
            *self = Arc::new(new_data);
        }
    }

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

    fn child_data(&self) -> Option<&T> {
        self.iter().next()
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
        let (major_spacing, minor_spacing) = match axis {
            Axis::Vertical => (
                self.vertical_spacing.resolve(env),
                self.horizontal_spacing.resolve(env),
            ),
            Axis::Horizontal => (
                self.horizontal_spacing.resolve(env),
                self.vertical_spacing.resolve(env),
            ),
        };
        let mut major_pos = 0.0;
        let mut minor_pos = 0.0;
        let mut paint_rect = Rect::ZERO;
        // let child_bc = constraints(axis, bc, 0., f64::INFINITY);
        // I don't know if this is the right way to go. I would assume a grid is
        // used in a Scroll and that would provide the infinite constraints if necessary
        // otherwise the scroll will be locked to an axis and provide concrete constraints
        // on that axis
        // this has to use axis.constraints function but it is private
        // reimplemented below for convenience
        let child_bc = constraints(axis, bc, 0., axis.major(bc.max()));

        // let child_bc = constraints(axis, bc, 0., );

        let minor_axis_count = match self.minor_axis_count {
            MinorAxisCount::Wrap => {
                let minor_len = axis.minor(bc.max());
                let child_size = match self.children.last_mut() {
                    Some(child) => {
                        let size = child.layout(ctx, &child_bc, data.child_data().unwrap(), env);
                        size
                    }
                    None => Size::ZERO,
                };
                if child_size == Size::ZERO {
                    // TODO: this should be zero, but i'm making it one to avoid divide by zero
                    1
                } else {
                    (minor_len / axis.minor(child_size)).floor() as usize
                }
            }
            MinorAxisCount::Count(count) => count as usize,
        };

        let mut children = self.children.iter_mut();

        data.row(
            |child_data, idx| {
                let child = match children.next() {
                    Some(child) => child,
                    None => return,
                };

                let child_size = child.layout(ctx, &child_bc, child_data, env);
                let child_pos: Point = axis.pack(major_pos, minor_pos).into();
                child.set_origin(ctx, child_data, env, child_pos);
                paint_rect = paint_rect.union(child.paint_rect());

                if (idx + 1) % minor_axis_count == 0 {
                    // TODO: have to correct overshoot
                    major_pos += axis.major(child_size) + major_spacing;
                    minor_pos = 0.;
                } else {
                    minor_pos += axis.minor(child_size) + minor_spacing;
                }
                // TODO: have to correct overshoot
            },
            minor_axis_count,
        );
        // data.for_each(|child_data, idx| {
        //     let child = match children.next() {
        //         Some(child) => child,
        //         None => return,
        //     };

        //     let child_size = child.layout(ctx, &child_bc, child_data, env);
        //     let child_pos: Point = axis.pack(major_pos, minor_pos).into();
        //     child.set_origin(ctx, child_data, env, child_pos);
        //     paint_rect = paint_rect.union(child.paint_rect());

        //     if (idx + 1) % minor_axis_count == 0 {
        //         // have to correct overshoot
        //         major_pos += axis.major(child_size) + spacing;
        //         minor_pos = 0.;
        //     } else {
        //         minor_pos += axis.minor(child_size) + spacing;
        //     }
        //     // have to correct overshoot
        // });

        // let my_size = bc.constrain(Size::from(axis.pack(major_pos, minor_pos)));
        // this should be correct, however the list widget uses above commented
        // code to get the widget size
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
