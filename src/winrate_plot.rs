use iced::Theme;

use iced_aksel::{Axis, Measure, Plot, PlotData, PlotPoint, State, Stroke, scale::Linear, stroke::StrokeStyle};

use crate::Message;

pub struct WinratePlot {
    pub data_points: Vec<f32>,
    pub state: State<&'static str, f32>,
}

impl WinratePlot {
    pub fn new() -> Self {
        let mut data_points = vec![0.0];
        // allocate now to prevent frequent allocations
        data_points.reserve_exact(300);

        let mut state = State::new();
        state.set_axis("x_id", Axis::new(Linear::new(0.0, 10.0), iced_aksel::axis::Position::Bottom));
        state.set_axis("y_id", Axis::new(Linear::new(0.0, 100.0), iced_aksel::axis::Position::Left));

        Self {
            data_points,
            state,
        }
    }

    pub fn update_plot(&mut self, (idx, winrate): (u64, f32)) {
        let idx = idx as usize;
        // updating already existing points
        if self.data_points.len() > idx {
            self.data_points[idx] = winrate;
            // for now we remove all further points as they are not up to date anymore
            self.data_points.truncate(idx+1);
        // inserting new points
        } else {
            // resize array with placeholder value
            self.data_points.resize(idx+1, -1.0);
            self.data_points[idx] = winrate;
        }
        let axis = self.state.axis_mut(&"x_id");
        axis.set_domain(0.0, usize::max(10, idx + idx/10) as f32);
    }
}

impl PlotData<f32, Message> for WinratePlot {
    fn draw(&self, plot: &mut Plot<f32, Message>, _theme: &Theme) {
        let mut skipped_values = false;
        let mut pred_point = if let Some(origin) = self.data_points.get(0) {
            PlotPoint::new(0.0, *origin)
        } else {
            PlotPoint::new(0.0, 0.0)
        };
        for (idx, point) in self.data_points.iter().enumerate() {
            if *point == -1.0 {
                skipped_values = true;
                continue;
            }

            let style = if skipped_values {
                skipped_values = false;
                StrokeStyle::Dashed { dash: 5.0, gap: 5.0 }
            } else {
                StrokeStyle::Solid
            };

            let point = PlotPoint::new(idx as f32, *point);
            plot.render(
                iced_aksel::shape::Line::new(pred_point, point,
                    Stroke::with_style(iced::color!(255,255,255), Measure::Plot(0.3), style)
            ));
            pred_point = point;
        }
    }
}
