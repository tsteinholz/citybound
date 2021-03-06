use kay::{ID, Recipient, Actor, Individual, Swarm, ActorSystem, Fate, CreateWith};
use descartes::{Band, Curve, Into2d, FiniteCurve, Path};
use ::core::geometry::{CPath, AnyShape};

use super::{SelectableStrokeRef, CurrentPlan};

#[derive(Actor, Compact, Clone)]
pub struct LaneStrokeSelectable {
    _id: Option<ID>,
    stroke_ref: SelectableStrokeRef,
    path: CPath,
}

impl LaneStrokeSelectable {
    pub fn new(stroke_ref: SelectableStrokeRef, path: CPath) -> Self {
        LaneStrokeSelectable {
            _id: None,
            stroke_ref: stroke_ref,
            path: path,
        }
    }
}

use super::AddToUI;
use ::core::ui::Add;

impl Recipient<AddToUI> for LaneStrokeSelectable {
    fn receive(&mut self, msg: &AddToUI) -> Fate {
        match *msg {
            AddToUI => {
                ::core::ui::UserInterface::id() <<
                Add::Interactable3d(self.id(),
                                    AnyShape::Band(Band::new(self.path.clone(), 5.0)),
                                    1);
                Fate::Live
            }
        }
    }
}

use super::ClearSelectables;
use ::core::ui::Remove;

impl Recipient<ClearSelectables> for LaneStrokeSelectable {
    fn receive(&mut self, msg: &ClearSelectables) -> Fate {
        match *msg {
            ClearSelectables => {
                ::core::ui::UserInterface::id() << Remove::Interactable3d(self.id());
                Fate::Die
            }
        }
    }
}

use ::core::ui::Event3d;
use super::{Select, Commit};

impl Recipient<Event3d> for LaneStrokeSelectable {
    fn receive(&mut self, msg: &Event3d) -> Fate {
        match *msg {
            Event3d::DragStarted { at } => {
                if let Some(selection_start) = self.path.project(at.into_2d()) {
                    CurrentPlan::id() <<
                    Select(self.stroke_ref,
                           (selection_start - 1.5).max(0.1),
                           (selection_start + 1.5).min(self.path.length() - 0.1));
                }
                Fate::Live
            }
            Event3d::DragOngoing { from, to } => {
                if let (Some(selection_start), Some(selection_end)) =
                    (self.path.project(from.into_2d()), self.path.project(to.into_2d())) {
                    let mut start = selection_start.min(selection_end);
                    let mut end = selection_end.max(selection_start);
                    if start < 10.0 {
                        start = 0.0
                    }
                    if end > self.path.length() - 10.0 {
                        end = self.path.length()
                    }
                    let mut offset = 0.0;
                    for segment in self.path.segments() {
                        let next_offset = offset + segment.length();
                        if start > offset - 5.0 && start < offset + 5.0 {
                            start = offset
                        }
                        if end > next_offset - 5.0 && end < next_offset + 5.0 {
                            end = next_offset
                        }
                        offset = next_offset;
                    }
                    CurrentPlan::id() <<
                    Select(self.stroke_ref,
                           start.min(end - 1.5).max(0.1),
                           end.max(start + 1.5).min(self.path.length() - 0.1));
                }
                Fate::Live
            }
            Event3d::DragFinished { to, .. } => {
                CurrentPlan::id() << Commit(true, to.into_2d());
                Fate::Live
            }
            _ => Fate::Live,
        }
    }
}


pub fn setup(system: &mut ActorSystem) {
    system.add_individual(Swarm::<LaneStrokeSelectable>::new());
    system.add_inbox::<CreateWith<LaneStrokeSelectable, AddToUI>, Swarm<LaneStrokeSelectable>>();
    system.add_inbox::<ClearSelectables, Swarm<LaneStrokeSelectable>>();
    system.add_inbox::<Event3d, Swarm<LaneStrokeSelectable>>();
}
