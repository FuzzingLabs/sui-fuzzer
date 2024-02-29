use std::sync::{Arc, RwLock};

use ratatui::prelude::*;

use crate::{detector::detector::AvailableDetector, fuzzer::stats::Stats, mutator::types::Type};

use super::{
    detectors_widget::DetectorWidget, global_stats_widget::GlobalStatsWidget,
    graphs_widget::GraphsWidget, target_infos_widget::TargetInfosWidget,
    worker_stats_widget::WorkerStatsWidget,
};

pub struct StatsWidget {
    global_stats_widget: GlobalStatsWidget,
    target_widget: TargetInfosWidget,
    worker_widget: WorkerStatsWidget,
    graphs_widget: GraphsWidget,
    detectors_widget: DetectorWidget,
}

impl StatsWidget {
    pub fn new(
        seed: u64,
        max_coverage: usize,
        target_module: &str,
        target_function: &str,
        target_parameters: &Vec<Type>,
    ) -> Self {
        Self {
            global_stats_widget: GlobalStatsWidget::new(seed, max_coverage),
            target_widget: TargetInfosWidget::new(
                target_module.to_string(),
                target_function.to_string(),
                target_parameters.to_vec(),
            ),
            worker_widget: WorkerStatsWidget::new(),
            graphs_widget: GraphsWidget::new(),
            detectors_widget: DetectorWidget::new(),
        }
    }

    pub fn render<B>(
        &mut self,
        frame: &mut Frame<B>,
        area: Rect,
        stats: &Stats,
        tab_index: usize,
        threads_stats_idx: usize,
        threads_stats: &Vec<Arc<RwLock<Stats>>>,
        detectors: &Option<Vec<AvailableDetector>>,
        use_state: bool,
    ) where
        B: Backend,
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .margin(1)
            .direction(Direction::Horizontal)
            .split(area);

        let stats_rects = Layout::default()
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .direction(Direction::Vertical)
            .split(chunks[0]);

        let rects_global_stats_detector = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .direction(Direction::Horizontal)
            .split(stats_rects[0]);

        let rects_stats_global_worker = Layout::default()
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .direction(Direction::Vertical)
            .split(rects_global_stats_detector[0]);

        // Render global stats widget
        self.global_stats_widget
            .render(frame, rects_stats_global_worker[0], stats, use_state);

        // Render worker stats widget
        self.worker_widget.render(
            frame,
            rects_stats_global_worker[1],
            threads_stats_idx,
            &threads_stats[threads_stats_idx],
        );

        // Render target info widget
        self.target_widget.render(frame, stats_rects[1]);

        // Render detectors widget
        self.detectors_widget
            .render(frame, rects_global_stats_detector[1], detectors);

        // Render graph widget
        self.graphs_widget
            .render(frame, chunks[1], stats, tab_index);
    }
}
