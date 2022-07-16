use crate::engine_commands::*;
use std::sync::{Arc, Mutex};
use druid::widget::prelude::*;
use druid::{ Data, Lens, Widget,};
use druid::widget::Button;
use druid::widget::Flex;

use plotters::prelude::*;
use plotters_piet::PietBackend;


#[derive(Clone, Data, Lens)]
pub struct Engine {
    //tuple is x coord and y value (x, y)
}


impl Widget<crate::RootState> for Engine {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut crate::RootState, _env: &Env) {
        match event {
            Event::Command(c) => {
                if c.get(crate::selectors::ANALYZE_TIMER_TOKEN).is_some() {
                    let mut analyze = data.analyze_info.lock().unwrap();
                    *analyze = data.engine.lock().expect("couldn't get engine").read_info().expect("failed to parse info");
                    data.analyze_timer_token = Arc::new(Some(ctx.request_timer(TIMER_INTERVAL.clone())));
                    ctx.request_paint();
                }
            },
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &crate::RootState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &crate::RootState, _data: &crate::RootState, _env: &Env) {
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &crate::RootState, _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &crate::RootState, _env: &Env) {
        let region = ctx.region();
        let region = region.bounding_box().contained_rect_with_aspect_ratio(1.2);
        let origin = (region.origin().x as u32, region.origin().y as u32);
        let root = PietBackend::new(ctx.render_ctx, (origin, region.width() as u32, region.height() as u32)).unwrap().into_drawing_area();
        
        root.fill(&WHITE).unwrap();
        let root = root.margin(10, 10, 10, 10);
        let points = data.winrate_points.lock().unwrap();
        let size = if points.len() < 50 {
            50.0
        } else {
            points.len() as f32
        };
        // After this point, we should be able to draw construct a chart context
        let mut chart = ChartBuilder::on(&root)
            // Set the caption of the chart
            .caption("Winrate for black", ("sans-serif", 6).into_font())
            // Set the size of the label region
            .x_label_area_size(20)
            .y_label_area_size(40)
            // Finally attach a coordinate on the drawing area and make a chart context
            .build_cartesian_2d(0f32..size, 0f32..100f32).unwrap();

        // Then we can draw a mesh
        chart
            .configure_mesh()
            // We can customize the maximum number of labels allowed for each axis
            .x_labels(5)
            .y_labels(5)
            // We can also change the format of the label text
            .y_label_formatter(&|x| format!("{:.3}", x))
            .draw().unwrap();

        // And we can draw something in the drawing area
        chart.draw_series(LineSeries::new(
            points.clone(),
            &RED,
        )).unwrap();
        // Similarly, we can draw point series
        //chart.draw_series(PointSeries::of_element(
        //    points.clone(),
        //    5,
        //    &RED,
        //    &|c, s, st| {
        //        return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
        //        + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
        //        + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
        //    },
        //)).unwrap();
        root.present().unwrap();
    }
}

impl Engine {
    pub fn build_engine_tab() -> impl Widget<crate::RootState> {
        Flex::column()
        .with_flex_child(Engine {}, 0.8)
        .with_flex_child(Flex::row()
        .with_flex_child(Button::new("Analyze").on_click(|ctx, data: &mut crate::RootState, _| {
            let mut engine = data.engine.lock().expect("couldn't get engine");
            match engine.send_command(COMMAND_ANALYZE.clone()).unwrap() {
                libgtp::Answer::Response(_) => {
                    let mut state = data.engine_state.lock().unwrap();
                    let _ = state.consume(&crate::EngineStateInput::StartAnalyze);
                    let mut analyze = data.analyze_info.lock().unwrap();
                    *analyze = engine.read_info().expect("failed to parse info I guess");
                    data.analyze_timer_token = Arc::new(Some(ctx.request_timer(TIMER_INTERVAL.clone())));
                },
                libgtp::Answer::Failure(f) => eprintln!("{}", f),
                _ => unreachable!(),
            };
        }), 0.1)
        .with_flex_child(Button::new("Stop").on_click(|_, data: &mut crate::RootState, _| {
            match data.engine.lock().expect("couldn't get engine").send_command(COMMAND_STOP.clone()).unwrap() {
                libgtp::Answer::Response(_) => {
                    let mut state = data.engine_state.lock().unwrap();
                    let _ = state.consume(&crate::EngineStateInput::StopAnalyze);
                    data.analyze_timer_token = Arc::new(None);
                },
                libgtp::Answer::Failure(f) => { eprintln!("{}", f); },
                _ => unreachable!(),
            };}), 0.1), 0.2)
    }

    pub fn engine_startup() -> libgtp::Controller {
        let mut controller = libgtp::Controller::new("./KataGo/katago", &["gtp", "-model", "./KataGo/model.bin.gz", "-config", "./KataGo/default_gtp.cfg"]);
        controller.send_command(COMMAND_RULES_JAPANESE.clone()).unwrap();
        controller.send_command(COMMAND_KOMI.clone()).unwrap();
        controller.send_command(COMMAND_CLEARBOARD.clone()).unwrap();
        controller
    }
}
